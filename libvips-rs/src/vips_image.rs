use std::{
    ffi::{c_void, CString},
    mem::transmute,
    os::raw::c_char,
    ptr::null_mut,
};

use crate::vips_source_custom::*;
use crate::vips_target_custom::*;
pub struct VipsImage<'a> {
    pub vips_source: VipsSourceCustom<'a>,
    pub(crate) vips_image: *mut libvips_sys::VipsImage,
}

unsafe impl<'a> Send for VipsImage<'a> {}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        if !self.vips_image.is_null() {
            unsafe {
                libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);
            }
        }
    }
}

impl<'a> VipsImage<'a> {
    pub fn thumbnail(&mut self, width: i32) {
        let out_ptr = null_mut::<libvips_sys::VipsImage>();

        unsafe {
            libvips_sys::vips_thumbnail_image(
                self.vips_image,
                transmute(&out_ptr),
                width,
                null_mut::<*const c_void>(),
            );

            libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);
        }

        self.vips_image = out_ptr;
    }

    pub fn resize(&mut self, vscale: f64) {
        let out_ptr = null_mut::<libvips_sys::VipsImage>();

        unsafe {
            libvips_sys::vips_resize(
                self.vips_image,
                transmute(&out_ptr),
                vscale,
                null_mut::<*const c_void>(),
            );

            libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);
        }

        self.vips_image = out_ptr;
    }

    pub fn write_to_target(&self, target: &VipsTargetCustom, suffix: &str) -> bool {
        let suffix_cstr = CString::new(suffix).unwrap();
        let r = unsafe {
            libvips_sys::vips_image_write_to_target(
                self.vips_image,
                suffix_cstr.as_ptr(),
                libvips_sys::g_type_cast(
                    target.vips_target_custom,
                    libvips_sys::vips_target_get_type(),
                ),
                null_mut::<*const c_void>(),
            )
        };

        r == 0
    }
}

pub fn thumbnail_from_source(source: VipsSourceCustom, width: i32) -> VipsImage {
    let mut vi = VipsImage {
        vips_image: null_mut(),
        vips_source: source,
    };

    let out_ptr = null_mut::<libvips_sys::VipsImage>();
    unsafe {
        libvips_sys::vips_thumbnail_source(
            libvips_sys::g_type_cast(
                vi.vips_source.vips_source_custom,
                libvips_sys::vips_source_get_type(),
            ),
            transmute(&out_ptr),
            width,
            null_mut::<*const c_void>(),
        );
    }
    vi.vips_image = out_ptr;

    vi
}

pub fn new_image_from_source(source: VipsSourceCustom) -> VipsImage {
    VipsImage {
        vips_image: unsafe {
            libvips_sys::vips_image_new_from_source(
                libvips_sys::g_type_cast(
                    source.vips_source_custom,
                    libvips_sys::vips_source_get_type(),
                ),
                "\0".as_ptr() as *const c_char,
                "access\0".as_ptr() as *const c_char,
                libvips_sys::VipsAccess::VIPS_ACCESS_SEQUENTIAL,
                null_mut::<*const c_void>(),
            )
        },
        vips_source: source,
    }
}
