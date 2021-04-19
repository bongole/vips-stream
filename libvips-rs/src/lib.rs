use std::{env, ffi::{CStr}, os::raw::c_char, sync::Once};

mod vips_image;
mod vips_source_custom;
mod vips_target_custom;

pub use vips_image::*;
pub use vips_source_custom::*;
pub use vips_target_custom::*;

static INIT: Once = Once::new();
static mut INIT_VAL: i32 = 0;

pub fn init() -> i32 {
    INIT.call_once(|| {
        if env::var("VIPS_MIN_STACK_SIZE").is_err() {
            env::set_var("VIPS_MIN_STACK_SIZE", "2m");
        }

        unsafe {
            INIT_VAL = libvips_sys::vips_init("libvips-rs\0".as_ptr() as *const c_char);
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


#[inline]
fn to_bool(i: i32) -> bool {
    i == 1
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

pub fn error() -> String {
    unsafe {
        let s = libvips_sys::vips_error_buffer();
        CStr::from_ptr(s).to_string_lossy().to_string()
    }
}

pub fn clear_error() {
    unsafe { libvips_sys::vips_error_clear() }
}

pub fn thread_shutdown() {
    unsafe { libvips_sys::vips_thread_shutdown() }
}

pub fn set_concurrency(c: i32) {
    unsafe { libvips_sys::vips_concurrency_set(c) }
}

pub fn concurrency() -> i32 {
    unsafe { libvips_sys::vips_concurrency_get() }
}
