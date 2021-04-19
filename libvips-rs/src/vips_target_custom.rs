use std::{ffi::c_void, mem::transmute, os::raw::c_char, ptr::{null_mut, slice_from_raw_parts}};

pub type WriteHandler = dyn FnMut(&[u8]) -> i64 + Send + 'static;
pub type FinishHandler = dyn FnMut() + Send + 'static;
pub struct VipsTargetCustom {
    pub(crate) vips_target_custom: *mut libvips_sys::VipsTargetCustom,
    pub(crate) write_handler: (Option<u64>, Option<Box<WriteHandler>>),
    pub(crate) finish_handler: (Option<u64>, Option<Box<FinishHandler>>)
}

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

                libvips_sys::g_object_unref(target);
            }
        }
    }
}

impl VipsTargetCustom {
    pub fn set_on_write<F>(&mut self, f: F)
    where
        F: FnMut(&[u8]) -> i64 + Send + 'static,
    {
        let closure = Box::new(f);

        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn write_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> libvips_sys::gint64 {
                let this: &mut VipsTargetCustom = std::mem::transmute(data);
                if let Some(callback) = this.write_handler.1.as_mut() {
                    let buf = slice_from_raw_parts(buf as *const u8, buf_len as usize);
                    callback(buf.as_ref().unwrap())
                } else {
                    0
                }
            }

            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "write\0".as_ptr() as *const c_char,
                Some(transmute(write_wrapper as *const fn())),
                self as *const _ as libvips_sys::gpointer,
            )
        };

        self.write_handler = (Some(handler_id), Some(closure));
    }

    pub fn set_on_finish<F>(&mut self, f: F)
    where
        F: FnMut() + Send + 'static,
    {
        let closure = Box::new(f);

        let handler_id = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn finish_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                data: *mut c_void,
            ) {
                let this: &mut VipsTargetCustom = std::mem::transmute(data);
                if let Some(callback) = this.finish_handler.1.as_mut() {
                    callback()
                }
            }

            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "finish\0".as_ptr() as *const c_char,
                Some(transmute(finish_wrapper as *const fn())),
                self as *const _ as libvips_sys::gpointer,
            )
        };

        self.finish_handler = (Some(handler_id), Some(closure));
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
        write_handler: (None, Default::default()),
        finish_handler: (None, Default::default()),
    };

    unsafe {
        let vips_target_ptr = libvips_sys::vips_target_custom_new();
        vtc.vips_target_custom = vips_target_ptr;
    }

    vtc
}
