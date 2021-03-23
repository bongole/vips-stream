use std::{env::set_var, env::var, ffi::{CStr, CString}, ptr::null_mut, sync::Once};

#[derive(Debug)]
pub struct VipsImage<'a> {
    pub(crate) vips_image: *mut libvips_sys::VipsImage,
    pub(crate) vips_source: &'a VipsSourceCustom,
}

#[derive(Debug)]
pub struct VipsSourceCustom {
    pub(crate) vips_source_custom: *mut libvips_sys::VipsSourceCustom,
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

static INIT: Once = Once::new();
static mut INIT_RET: i32 = 0;

pub fn init() -> i32 {
    unsafe {
        INIT.call_once(|| {
            if var("VIPS_MIN_STACK_SIZE").is_err() {
                set_var("VIPS_MIN_STACK_SIZE", "2m");
            }

            let init_name = CString::new("libvips-rust").unwrap();
            INIT_RET = libvips_sys::vips_init(init_name.as_ptr());
        });

        INIT_RET
    }
}

pub fn version() -> String {
    unsafe {
        let major = libvips_sys::vips_version(0);
        let minor = libvips_sys::vips_version(1);
        let patch = libvips_sys::vips_version(2);
        format!("{}.{}.{}", major, minor, patch)
    }
}

#[inline]
fn to_bool(i: i32) -> bool {
    matches!(i, 1)
}

#[inline]
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
    };

    unsafe {
        let vips_source_ptr = libvips_sys::vips_source_custom_new();
        vsc.vips_source_custom = vips_source_ptr;
    }

    vsc
}

pub fn new_image_from_source(source: &VipsSourceCustom) -> VipsImage {
    let mut vi = VipsImage { vips_image: null_mut(), vips_source: source };

    let empty_str = CString::new("").unwrap();
    unsafe {
        let vips_image_ptr = libvips_sys::vips_image_new_from_source(source.vips_source_custom as *mut libvips_sys::VipsSource, empty_str.as_ptr(), null_mut::<i32>());
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
    unsafe {
        libvips_sys::vips_error_clear()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_init() {
        let b = crate::init();
        assert_eq!(0, b);
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
    fn test_new_image() {
        crate::init();
        let source = crate::new_source_custom();
        let r = crate::new_image_from_source(&source);
        assert!(!r.vips_image.is_null());
    }

}
