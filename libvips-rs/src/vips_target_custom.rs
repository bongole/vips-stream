use std::{
    ffi::c_void,
    mem::transmute,
    ptr::{null_mut, slice_from_raw_parts},
    os::raw::c_char
};

pub type WriteHandlerType = (Option<u64>, Option<Box<dyn FnMut(&[u8]) -> i64>>);
pub type FinishHandlerType = (Option<u64>, Option<Box<dyn FnMut()>>);
pub struct VipsTargetCustom {
    pub(crate) vips_target_custom: *mut libvips_sys::VipsTargetCustom,
    pub(crate) write_handler: WriteHandlerType,
    pub(crate) finish_handler: FinishHandlerType,
}

unsafe impl Sync for VipsTargetCustom {}
unsafe impl Send for VipsTargetCustom {}

impl Drop for VipsTargetCustom {
    fn drop(&mut self) {
        unsafe {
            if !self.vips_target_custom.is_null() {
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
}

impl VipsTargetCustom {
    pub fn set_on_write<F>(&mut self, f: F)
    where
        F: FnMut(&[u8]) -> i64,
        F: 'static,
    {
        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn write_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> libvips_sys::gint64 {
                let this: &mut VipsTargetCustom = &mut *(data as *mut VipsTargetCustom);
                if let Some(ref mut callback) = this.write_handler.1 {
                    let buf = slice_from_raw_parts(buf as *const u8, buf_len as usize);
                    callback(&*buf)
                } else {
                    0
                }
            }

            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "write\0".as_ptr() as *const c_char,
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

            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "finish\0".as_ptr() as *const c_char,
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

pub fn new_target_custom() -> VipsTargetCustom {
    let mut vtc = VipsTargetCustom {
        vips_target_custom: null_mut(),
        write_handler: (None, None),
        finish_handler: (None, None),
    };

    unsafe {
        let vips_target_ptr = libvips_sys::vips_target_custom_new();
        vtc.vips_target_custom = vips_target_ptr;
    }

    vtc
}
