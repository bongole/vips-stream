use std::{ffi::c_void, ffi::CString, ptr::slice_from_raw_parts_mut, mem::transmute};

pub type ReadHandlerType = (Option<u64>, Option<Box<dyn FnMut(&mut [u8]) -> usize>>);
pub struct VipsSourceCustom {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
    pub(crate) read_handler: ReadHandlerType,
}

impl Drop for VipsSourceCustom {
    fn drop(&mut self) {
        unsafe {
            let source = self.vips_source_custom as libvips_sys::gpointer;

            if let Some(handler_id) = self.read_handler.0 {
                libvips_sys::g_signal_handler_disconnect(source, handler_id);
            }

            libvips_sys::g_object_unref(source);
        }
    }
}

impl VipsSourceCustom {
    pub fn set_on_read<F>(&mut self, f: F)
    where
        F: FnMut(&mut [u8]) -> usize,
        F: 'static,
    {
        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn read_wrapper(
                _source: *mut libvips_sys::VipsSourceCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> usize {
                let this: &mut VipsSourceCustom = &mut *(data as *mut VipsSourceCustom);
                if let Some(ref mut callback) = this.read_handler.1 {
                    let buf = slice_from_raw_parts_mut(buf as *mut u8, buf_len as usize);
                    callback(&mut *buf)
                } else {
                    0
                }
            }

            let read_k = CString::new("read").unwrap();
            libvips_sys::g_signal_connect(
                self.vips_source_custom as libvips_sys::gpointer,
                read_k.as_ptr(),
                Some(transmute(read_wrapper as *const fn())),
                self as *mut _ as libvips_sys::gpointer,
            )
        };

        self.read_handler = (Some(handler_id), Some(Box::new(f)));
    }

    pub fn read_position(&self) -> i64 {
        unsafe {
            let p = (*self.vips_source_custom).parent_object;
            p.read_position
        }
    }
}
