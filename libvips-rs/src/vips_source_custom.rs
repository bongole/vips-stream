use std::{ffi::c_void, mem::transmute, os::raw::c_char, pin::Pin, ptr::slice_from_raw_parts_mut};

pub struct VipsSourceCustom<'a> {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
    pub(crate) read_handler: Option<(u64, Pin<Box<Box<dyn FnMut(&mut [u8]) -> i64 + 'a>>>)>,
}

unsafe impl<'a> Send for VipsSourceCustom<'a> {}

impl<'a> Drop for VipsSourceCustom<'a> {
    fn drop(&mut self) {
        if !self.vips_source_custom.is_null() {
            let source = self.vips_source_custom as libvips_sys::gpointer;

            if let Some((handler_id, _)) = self.read_handler {
                unsafe {
                    libvips_sys::g_signal_handler_disconnect(source, handler_id);
                }
            }

            unsafe {
                libvips_sys::g_object_unref(source);
            }
        }
    }
}

impl<'a> VipsSourceCustom<'a> {
    pub fn set_on_read<F>(&mut self, f: F)
    where
        F: FnMut(&mut [u8]) -> i64 + Send + 'a
    {
        extern "C" fn read_wrapper(
            _source: *mut libvips_sys::VipsSourceCustom,
            buf: *mut c_void,
            buf_len: libvips_sys::gint64,
            data: *mut c_void,
        ) -> libvips_sys::gint64 {
            let cb = data as *mut Box<dyn FnMut(&mut [u8]) -> i64 >;
            let buf = slice_from_raw_parts_mut(buf as *mut _, buf_len as _);
            unsafe { (*cb)(buf.as_mut().unwrap()) }
        }

        let mut cb: Pin<Box<Box<dyn FnMut(&mut [u8]) -> i64 + 'a>>> = Box::pin(Box::new(f));

        let handler_id = unsafe {
            libvips_sys::g_signal_connect(
                self.vips_source_custom as libvips_sys::gpointer,
                "read\0".as_ptr() as *const c_char,
                Some(transmute(read_wrapper as *const fn())),
                &mut *cb as *mut _ as *mut c_void,
            )
        };

        self.read_handler = Some((handler_id, cb));
    }

    pub fn read_position(&self) -> i64 {
        unsafe {
            let p = (*self.vips_source_custom).parent_object;
            p.read_position
        }
    }
}

pub fn new_source_custom<'a>() -> VipsSourceCustom<'a> {
    VipsSourceCustom {
        vips_source_custom: unsafe { libvips_sys::vips_source_custom_new() },
        read_handler: Default::default(),
    }
}
