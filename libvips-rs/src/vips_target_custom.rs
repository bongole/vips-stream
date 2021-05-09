use std::{ffi::c_void, mem::transmute, os::raw::c_char, pin::Pin, ptr::slice_from_raw_parts};

pub type OnWriteHandler = dyn FnMut(&[u8]) -> i64;
pub type OnFinishHandler = dyn FnMut();

pub struct VipsTargetCustom {
    pub(crate) vips_target_custom: *mut libvips_sys::VipsTargetCustom,
    pub(crate) write_handler: Option<(u64, Pin<Box<Box<OnWriteHandler>>>)>,
    pub(crate) finish_handler: Option<(u64, Pin<Box<Option<Box<OnFinishHandler>>>>)>,
}

unsafe impl Send for VipsTargetCustom {}

impl Drop for VipsTargetCustom {
    fn drop(&mut self) {
        if !self.vips_target_custom.is_null() {
            let target = self.vips_target_custom as libvips_sys::gpointer;

            if let Some((handler_id, _)) = self.write_handler {
                unsafe {
                    libvips_sys::g_signal_handler_disconnect(target, handler_id);
                }
            }

            if let Some((handler_id, _)) = self.finish_handler {
                unsafe {
                    libvips_sys::g_signal_handler_disconnect(target, handler_id);
                }
            }

            unsafe {
                libvips_sys::g_object_unref(target);
            }
        }
    }
}

impl VipsTargetCustom {
    pub fn set_on_write<F>(&mut self, f: F)
    where
        F: FnMut(&[u8]) -> i64 + Send + 'static
    {
        extern "C" fn write_wrapper(
            _target: *mut libvips_sys::VipsTargetCustom,
            buf: *mut c_void,
            buf_len: libvips_sys::gint64,
            data: *mut c_void,
        ) -> libvips_sys::gint64 {
            let cb = data as *mut Box<OnWriteHandler>;
            let buf = slice_from_raw_parts(buf as *const u8, buf_len as usize);
            unsafe { (*cb)(buf.as_ref().unwrap()) }
        }

        let mut cb: Pin<Box<Box<OnWriteHandler>>> = Box::pin(Box::new(f));

        let handler_id = unsafe {
            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "write\0".as_ptr() as *const c_char,
                Some(transmute(write_wrapper as *const fn())),
                &mut *cb as *mut _ as *mut c_void,
            )
        };

        self.write_handler = Some((handler_id, cb));
    }

    pub fn set_on_finish<F>(&mut self, f: F)
    where
        F: FnMut() + Send + 'static,
    {
        extern "C" fn finish_wrapper(
            _target: *mut libvips_sys::VipsTargetCustom,
            data: *mut c_void,
        ) {
            let cb = data as *mut Option<Box<OnFinishHandler>>;
            unsafe {
                if let Some(ref mut f) = *cb {
                    f();
                    *cb = None;
                }
            }
        }

        let mut cb: Pin<Box<Option<Box<OnFinishHandler>>>> = Box::pin(Some(Box::new(f)));

        let handler_id = unsafe {
            libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "finish\0".as_ptr() as *const c_char,
                Some(transmute(finish_wrapper as *const fn())),
                &mut *cb as *mut _ as *mut c_void,
            )
        };

        self.finish_handler = Some((handler_id, cb));
    }

    pub fn is_finished(&self) -> bool {
        unsafe {
            let parent = (*self.vips_target_custom).parent_object;
            parent.finished == 1
        }
    }
}

pub fn new_target_custom() -> VipsTargetCustom {
    VipsTargetCustom {
        vips_target_custom: unsafe { libvips_sys::vips_target_custom_new() },
        write_handler: Default::default(),
        finish_handler: Default::default(),
    }
}
