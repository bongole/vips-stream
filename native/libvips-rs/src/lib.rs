use std::{
    env,
    ffi::{CStr, CString},
    os::raw::c_char,
    ptr::null_mut,
    sync::Once,
};

mod vips_source_custom;
use vips_source_custom::*;

mod vips_target_custom;
use vips_target_custom::*;

mod vips_image;
use vips_image::*;

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
        _vips_source: source,
    };

    unsafe {
        let empty_str = CString::new("").unwrap();
        let vips_image_ptr = libvips_sys::vips_image_new_from_source(
            libvips_sys::g_type_cast(
                source.vips_source_custom,
                libvips_sys::vips_source_get_type(),
            ),
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
        write_handler: (None, None),
        finish_handler: (None, None),
    };

    unsafe {
        let vips_target_ptr = libvips_sys::vips_target_custom_new();
        vtc.vips_target_custom = vips_target_ptr;
    }

    vtc
}

static INIT: Once = Once::new();
static mut INIT_VAL: i32 = 0;

pub fn init() -> i32 {
    INIT.call_once(|| {
        if env::var("VIPS_MIN_STACK_SIZE").is_err() {
            env::set_var("VIPS_MIN_STACK_SIZE", "2m");
        }

        unsafe {
            let init_name = CString::new("libvips-rs").unwrap();
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
    i == 1
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

pub fn error() -> String {
    unsafe {
        let s = libvips_sys::vips_error_buffer();
        CStr::from_ptr(s).to_string_lossy().to_string()
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