use std::{ffi::c_void, ffi::CString, mem::transmute, ptr::slice_from_raw_parts};

pub type WriteHandlerType = (Option<u64>, Option<Box<dyn FnMut(&[u8]) -> usize>>);
pub type FinishHandlerType = (Option<u64>, Option<Box<dyn FnMut()>>);
pub struct VipsTargetCustom {
    pub(crate) vips_target_custom: *mut libvips_sys::VipsTargetCustom,
    pub(crate) write_handler: WriteHandlerType,
    pub(crate) finish_handler: FinishHandlerType,
}

impl Drop for VipsTargetCustom {
    fn drop(&mut self) {
        unsafe {
            let target = self.vips_target_custom as libvips_sys::gpointer;

            if let Some(handler_id) = self.write_handler.0 {
                libvips_sys::g_signal_handler_disconnect(target, handler_id);
            }

            if let Some(handler_id) = self.finish_handler.0 {
                libvips_sys::g_signal_handler_disconnect(target, handler_id);
            }

            libvips_sys::g_object_unref(self.vips_target_custom as libvips_sys::gpointer);
        }
    }
}

impl VipsTargetCustom {
    pub fn set_on_write<F>(&mut self, f: F)
    where
        F: FnMut(&[u8]) -> usize,
        F: 'static,
    {
        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn write_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> usize {
                let this: &mut VipsTargetCustom = &mut *(data as *mut VipsTargetCustom);
                if let Some(ref mut callback) = this.write_handler.1 {
                    let buf = slice_from_raw_parts(buf as *const u8, buf_len as usize);
                    callback(&*buf)
                } else {
                    0
                }
            }

            let write_k = CString::new("write").unwrap();
            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                write_k.as_ptr(),
                Some(transmute(write_wrapper as *const fn())),
                self as *mut _ as libvips_sys::gpointer,
            )
        };

        self.write_handler = (Some(handler_id), Some(Box::new(f)));
    }

    pub fn set_on_finish<F>(&mut self, f: F)
    where
        F: FnMut(),
        F: 'static,
    {
        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn finish_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                data: *mut c_void,
            ) {
                let this: &mut VipsTargetCustom = &mut *(data as *mut VipsTargetCustom);
                if let Some(ref mut callback) = this.finish_handler.1 {
                    callback()
                }
            }

            let finish_k = CString::new("finish").unwrap();
            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                finish_k.as_ptr(),
                Some(transmute(finish_wrapper as *const fn())),
                self as *mut _ as libvips_sys::gpointer,
            )
        };

        self.finish_handler = (Some(handler_id), Some(Box::new(f)));
    }

    pub fn is_finished(&self) -> bool {
        unsafe {
            let parent = (*self.vips_target_custom).parent_object;
            parent.finished == 1
        }
    }
}
