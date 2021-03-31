use std::{
    ffi::{c_void, CString},
    mem::transmute,
    os::raw::c_char,
    ptr::null_mut,
};

use crate::vips_source_custom::*;
use crate::vips_target_custom::*;
pub struct VipsImage<'a> {
    pub(crate) vips_image: *mut libvips_sys::VipsImage,
    pub(crate) _vips_source: &'a VipsSourceCustom,
}

impl<'a> Drop for VipsImage<'a> {
    fn drop(&mut self) {
        unsafe {
            libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);
        }
    }
}

impl<'a> VipsImage<'a> {
    pub fn thumbnail(&mut self, width: i32) -> &mut Self {
        unsafe {
            let out_ptr = null_mut::<libvips_sys::VipsImage>();
            libvips_sys::vips_thumbnail_image(
                self.vips_image,
                transmute(&out_ptr),
                width,
                null_mut::<*const c_void>(),
            );

            libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);

            self.vips_image = out_ptr;
        }

        self
    }

    pub fn write_to_target(&self, target: &VipsTargetCustom, suffix: &str) -> bool {
        unsafe {
            let suffix_k = CString::new(suffix).unwrap();
            let r = libvips_sys::vips_image_write_to_target(
                self.vips_image,
                suffix_k.as_ptr(),
                libvips_sys::g_type_cast(
                    target.vips_target_custom,
                    libvips_sys::vips_target_get_type(),
                ),
                null_mut::<*const c_void>(),
            );

            r == 0
        }
    }
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
