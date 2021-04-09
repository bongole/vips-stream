use std::{
    ffi::c_void,
    mem::transmute,
    ptr::{null_mut, slice_from_raw_parts_mut},
    os::raw::c_char
};

pub type ReadHandlerType = (Option<u64>, Option<Box<dyn FnMut(&mut [u8]) -> i64>>);
pub struct VipsSourceCustom {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
    pub(crate) read_handler: ReadHandlerType,
}

unsafe impl Sync for VipsSourceCustom {}
unsafe impl Send for VipsSourceCustom {}

impl Drop for VipsSourceCustom {
    fn drop(&mut self) {
        unsafe {
            if !self.vips_source_custom.is_null() {
                let source = self.vips_source_custom as libvips_sys::gpointer;

                if let Some(handler_id) = self.read_handler.0 {
                    libvips_sys::g_signal_handler_disconnect(source, handler_id);
                }

                libvips_sys::g_object_unref(source);
            }
        }
    }
}

impl VipsSourceCustom {
    pub fn set_on_read<F>(&mut self, f: F)
    where
        F: FnMut(&mut [u8]) -> i64,
        F: 'static,
    {
        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn read_wrapper(
                _source: *mut libvips_sys::VipsSourceCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> libvips_sys::gint64 {
                let this: &mut VipsSourceCustom = &mut *(data as *mut VipsSourceCustom);
                if let Some(ref mut callback) = this.read_handler.1 {
                    let buf = slice_from_raw_parts_mut(buf as *mut u8, buf_len as usize);
                    callback(&mut *buf)
                } else {
                    0
                }
            }

            libvips_sys::g_signal_connect(
                self.vips_source_custom as libvips_sys::gpointer,
                "read\0".as_ptr() as *const c_char,
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

pub fn new_source_custom() -> VipsSourceCustom {
    let mut vsc = VipsSourceCustom {
        vips_source_custom: null_mut(),
        read_handler: (None, None),
    };

    unsafe {
        let vips_source_ptr = libvips_sys::vips_source_custom_new();
        vsc.vips_source_custom = vips_source_ptr;
    }

    vsc
}
