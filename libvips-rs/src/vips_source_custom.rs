use std::{
    ffi::c_void,
    mem::transmute,
    os::raw::c_char,
    ptr::{slice_from_raw_parts_mut},
};

pub(crate) struct OnReadClosureWrapper {
    pub(crate) closure: Box<dyn FnMut(&mut [u8]) -> i64 + Send + 'static>,
}

pub struct VipsSourceCustom {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
    pub(crate) read_handler: Option<(u64, *mut OnReadClosureWrapper)>,
}

unsafe impl Send for VipsSourceCustom {}

impl Drop for VipsSourceCustom {
    fn drop(&mut self) {
        if !self.vips_source_custom.is_null() {
            let source = self.vips_source_custom as libvips_sys::gpointer;

            if let Some((handler_id, wrapper_ptr)) = self.read_handler {
                unsafe {
                    libvips_sys::g_signal_handler_disconnect(source, handler_id);
                    let _ = Box::from_raw(wrapper_ptr as *mut OnReadClosureWrapper);
                }
            }

            unsafe {
                libvips_sys::g_object_unref(source);
            }
        }
    }
}

impl VipsSourceCustom {
    pub fn set_on_read<F>(&mut self, f: F)
    where
        F: FnMut(&mut [u8]) -> i64 + Send + 'static,
    {
        extern "C" fn read_wrapper(
            _source: *mut libvips_sys::VipsSourceCustom,
            buf: *mut c_void,
            buf_len: libvips_sys::gint64,
            data: *mut c_void,
        ) -> libvips_sys::gint64 {
            let wrapper = data as *mut OnReadClosureWrapper;
            let buf = slice_from_raw_parts_mut(buf as *mut _, buf_len as _);
            unsafe { ((*wrapper).closure)(buf.as_mut().unwrap()) }
        }

        let closure_ptr = Box::into_raw(Box::new(OnReadClosureWrapper {
            closure: Box::new(f),
        }));

        let handler_id = unsafe {
            libvips_sys::g_signal_connect(
                self.vips_source_custom as libvips_sys::gpointer,
                "read\0".as_ptr() as *const c_char,
                Some(transmute(read_wrapper as *const fn())),
                closure_ptr as _,
            )
        };

        self.read_handler = Some((handler_id, closure_ptr));
    }

    pub fn read_position(&self) -> i64 {
        unsafe {
            let p = (*self.vips_source_custom).parent_object;
            p.read_position
        }
    }
}

pub fn new_source_custom() -> VipsSourceCustom {
    VipsSourceCustom {
        vips_source_custom: unsafe { libvips_sys::vips_source_custom_new() },
        read_handler: Default::default(),
    }
}
