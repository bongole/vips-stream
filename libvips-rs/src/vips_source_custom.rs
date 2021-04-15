use std::{
    ffi::c_void,
    mem::transmute,
    os::raw::c_char,
    ptr::{null_mut, slice_from_raw_parts_mut},
    sync::{Arc, Mutex},
};

type ReadHandler = dyn FnMut(&mut [u8]) -> i64 + Send + 'static;
type ReadHandlerBox = Arc<Mutex<Box<ReadHandler>>>;
pub struct VipsSourceCustom {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
    pub read_handler: Option<(
        u64,
        ReadHandlerBox,
        *mut c_void,
    )>,
}

unsafe impl Send for VipsSourceCustom {}

impl Drop for VipsSourceCustom {
    fn drop(&mut self) {
        unsafe {
            if !self.vips_source_custom.is_null() {
                let source = self.vips_source_custom as libvips_sys::gpointer;

                if let Some((handler_id, _, leaked_box_ptr)) = self.read_handler {
                    libvips_sys::g_signal_handler_disconnect(source, handler_id);
                    let _ = Box::from_raw(leaked_box_ptr as *mut ReadHandlerBox);
                }

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
        let closure: ReadHandlerBox = Arc::new(Mutex::new(Box::new(f)));

        let (handler_id, leaked_box_ptr) = unsafe {
            #[allow(non_snake_case)]
            unsafe extern "C" fn read_wrapper(
                _source: *mut libvips_sys::VipsSourceCustom,
                buf: *mut c_void,
                buf_len: libvips_sys::gint64,
                data: *mut c_void,
            ) -> libvips_sys::gint64 {
                let leaked_box = Box::from_raw(data as *mut ReadHandlerBox);
                let read_size = match leaked_box.lock() {
                    Ok(mut callback) => {
                        let buf = slice_from_raw_parts_mut(buf as *mut u8, buf_len as usize);
                        callback(buf.as_mut().unwrap())
                    }
                    Err(_) => 0,
                };

                Box::leak(leaked_box);

                read_size
            }

            let leaked_box_ref = Box::leak(Box::new(closure.clone()));
            let handler_id = libvips_sys::g_signal_connect(
                self.vips_source_custom as libvips_sys::gpointer,
                "read\0".as_ptr() as *const c_char,
                Some(transmute(read_wrapper as *const fn())),
                leaked_box_ref as *mut _ as libvips_sys::gpointer,
            );

            (handler_id, leaked_box_ref as *mut _ as *mut c_void)
        };

        self.read_handler = Some((handler_id, closure, leaked_box_ptr));
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
        read_handler: Default::default()
    };

    unsafe {
        let vips_source_ptr = libvips_sys::vips_source_custom_new();
        vsc.vips_source_custom = vips_source_ptr;
    }

    vsc
}
