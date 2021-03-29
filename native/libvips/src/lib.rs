use std::{
    env::set_var,
    env::var,
    ffi::{c_void, CStr, CString},
    mem::transmute,
    os::raw::c_char,
    ptr::{null_mut, slice_from_raw_parts_mut},
    sync::Once,
};

pub struct VipsImage<'a> {
    pub(crate) vips_image: *mut libvips_sys::VipsImage,
    pub(crate) vips_source: &'a VipsSourceCustom,
}

type ReadHandlerType = (Option<u64>, Option<Box<dyn FnMut(&mut [u8]) -> usize>>);
pub struct VipsSourceCustom {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
    pub(crate) read_handler: ReadHandlerType,
}

#[derive(Debug)]
pub struct VipsTargetCustom {
    pub(crate) vips_target_custom: *mut libvips_sys::VipsTargetCustom,
}

impl Drop for VipsSourceCustom {
    fn drop(&mut self) {
        unsafe {
            libvips_sys::g_object_unref(self.vips_source_custom as libvips_sys::gpointer);
        }
    }
}

impl Drop for VipsTargetCustom {
    fn drop(&mut self) {
        unsafe {
            libvips_sys::g_object_unref(self.vips_target_custom as libvips_sys::gpointer);
        }
    }
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        unsafe {
            libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);
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
}

static INIT: Once = Once::new();
static mut INIT_VAL: i32 = 0;

pub fn init() -> i32 {
    INIT.call_once(|| {
        if var("VIPS_MIN_STACK_SIZE").is_err() {
            set_var("VIPS_MIN_STACK_SIZE", "2m");
        }

        unsafe {
            let init_name = CString::new("libvips-rust").unwrap();
            INIT_VAL = libvips_sys::vips_init(init_name.as_ptr());
        }
    });

    unsafe { INIT_VAL }
}

pub fn version() -> String {
    unsafe {
        let major = libvips_sys::vips_version(0);
        let minor = libvips_sys::vips_version(1);
        let patch = libvips_sys::vips_version(2);
        format!("{}.{}.{}", major, minor, patch)
    }
}

fn to_bool(i: i32) -> bool {
    matches!(i, 1)
}

fn to_int(b: bool) -> i32 {
    if b {
        1
    } else {
        0
    }
}

pub fn is_simd_enabled() -> bool {
    let b = unsafe { libvips_sys::vips_vector_isenabled() };
    to_bool(b)
}

pub fn set_simd_enabled(b: bool) {
    let bi = to_int(b);
    unsafe { libvips_sys::vips_vector_set_enabled(bi) }
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

pub fn new_image_from_source(source: &VipsSourceCustom) -> VipsImage {
    let mut vi = VipsImage {
        vips_image: null_mut(),
        vips_source: source,
    };

    unsafe {
        let empty_str = CString::new("").unwrap();
        let vips_image_ptr = libvips_sys::vips_image_new_from_source(
            source.vips_source_custom as *mut libvips_sys::VipsSource,
            empty_str.as_ptr(),
            null_mut::<*const c_char>(),
        );

        vi.vips_image = vips_image_ptr;
    }

    vi
}

pub fn new_target_custom() -> VipsTargetCustom {
    let mut vtc = VipsTargetCustom {
        vips_target_custom: null_mut(),
    };

    unsafe {
        let vips_target_ptr = libvips_sys::vips_target_custom_new();
        vtc.vips_target_custom = vips_target_ptr;
    }

    vtc
}

pub fn error() -> String {
    unsafe {
        let s = libvips_sys::vips_error_buffer();
        CStr::from_ptr(s).to_str().unwrap().to_string()
    }
}

pub fn clear_error() {
    unsafe { libvips_sys::vips_error_clear() }
}

pub fn set_concurrency(c: i32) {
    unsafe { libvips_sys::vips_concurrency_set(c) }
}

pub fn concurrency() -> i32 {
    unsafe { libvips_sys::vips_concurrency_get() }
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read};

    #[test]
    fn test_init() {
        let b = crate::init();
        assert_eq!(0, b);
    }

    #[test]
    fn test_concurrency() {
        crate::init();

        crate::set_concurrency(1);
        assert_eq!(1, crate::concurrency());

        crate::set_concurrency(0);
        assert!(0 != crate::concurrency());
    }

    #[test]
    fn test_set_simd() {
        crate::init();

        crate::set_simd_enabled(true);
        assert_eq!(true, crate::is_simd_enabled());

        crate::set_simd_enabled(false);
        assert_eq!(false, crate::is_simd_enabled());
    }

    #[test]
    fn test_version() {
        crate::init();
        let version = crate::version();
        assert!(!"".eq(&version));
    }
    #[test]
    fn test_new_source_custom() {
        crate::init();
        let r = crate::new_source_custom();
        assert!(!r.vips_source_custom.is_null());
    }

    #[test]
    fn test_source_custom_set_on_read() {
        crate::init();
        let mut src = crate::new_source_custom();

        let file_path = format!("{}/test/test.jpg", env!("CARGO_MANIFEST_DIR"));
        let mut file = File::open(file_path).unwrap();

        src.set_on_read(move |buf| file.read(buf).unwrap());

        let vi = crate::new_image_from_source(&src);
        assert!(!vi.vips_image.is_null());
    }
}
