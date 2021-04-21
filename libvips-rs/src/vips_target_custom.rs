use std::{
    ffi::c_void,
    mem::transmute,
    os::raw::c_char,
    ptr::{null_mut, slice_from_raw_parts},
};

pub type OnWriteHandler = dyn FnMut(&[u8]) -> i64 + Send + 'static;
pub type OnFinishHandler = dyn FnOnce() + Send + 'static;
struct OnFinishClosureWrapper {
    pub(crate) closure: Option<Box<OnFinishHandler>>,
}
struct OnWriteClosureWrapper {
    pub(crate) closure: Box<OnWriteHandler>,
}
pub struct VipsTargetCustom {
    pub(crate) vips_target_custom: *mut libvips_sys::VipsTargetCustom,
    pub(crate) write_handler: Option<(u64, *mut c_void)>,
    pub(crate) finish_handler: Option<(u64, *mut c_void)>,
}

unsafe impl Send for VipsTargetCustom {}

impl Drop for VipsTargetCustom {
    fn drop(&mut self) {
        unsafe {
            if !self.vips_target_custom.is_null() {
                let target = self.vips_target_custom as libvips_sys::gpointer;

                if let Some((handler_id, leaked_closure_ptr)) = self.write_handler {
                    libvips_sys::g_signal_handler_disconnect(target, handler_id);
                    let _ = Box::from_raw(leaked_closure_ptr as *mut OnWriteClosureWrapper);
                }

                if let Some((handler_id, leaked_closure_ptr)) = self.finish_handler {
                    libvips_sys::g_signal_handler_disconnect(target, handler_id);
                    let _ = Box::from_raw(leaked_closure_ptr as *mut OnFinishClosureWrapper);
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
        let leaked_closure_ref = Box::leak(Box::new(OnWriteClosureWrapper {
            closure: Box::new(f),
        }));

        let r = unsafe {
            unsafe extern "C" fn write_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> libvips_sys::gint64 {
                let mut wrapper = Box::from_raw(data as *mut OnWriteClosureWrapper);
                let buf = slice_from_raw_parts(buf as *const u8, buf_len as usize);
                let r = (wrapper.closure)(buf.as_ref().unwrap());

                Box::leak(wrapper);

                r
            }

            let handler_id = libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "write\0".as_ptr() as *const c_char,
                Some(transmute(write_wrapper as *const fn())),
                leaked_closure_ref as *const _ as libvips_sys::gpointer,
            );

            (handler_id, leaked_closure_ref as *mut _ as *mut c_void)
        };

        self.write_handler = Some(r);
    }

    pub fn set_on_finish<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let leaked_closure_ref = Box::leak(Box::new(OnFinishClosureWrapper {
            closure: Some(Box::new(f)),
        }));

        let r = unsafe {
            unsafe extern "C" fn finish_wrapper(
                _target: *mut libvips_sys::VipsTargetCustom,
                data: *mut c_void,
            ) {
                let mut wrapper = Box::from_raw(data as *mut OnFinishClosureWrapper);
                if let Some(closure) = wrapper.closure {
                    closure();
                    wrapper.closure = None;
                }

                Box::leak(wrapper);
            }

            let handler_id = libvips_sys::g_signal_connect(
                self.vips_target_custom as libvips_sys::gpointer,
                "finish\0".as_ptr() as *const c_char,
                Some(transmute(finish_wrapper as *const fn())),
                leaked_closure_ref as *const _ as libvips_sys::gpointer,
            );

            (handler_id, leaked_closure_ref as *mut _ as *mut c_void)
        };

        self.finish_handler = Some(r);
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
        write_handler: Default::default(),
        finish_handler: Default::default(),
    };

    unsafe {
        let vips_target_ptr = libvips_sys::vips_target_custom_new();
        vtc.vips_target_custom = vips_target_ptr;
    }

    vtc
}
