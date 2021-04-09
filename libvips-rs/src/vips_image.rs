use std::{ffi::{c_void, CString}, mem::transmute, os::raw::c_char, ptr::null_mut};

use crate::vips_source_custom::*;
use crate::vips_target_custom::*;
pub struct VipsImage {
    pub(crate) vips_image: *mut libvips_sys::VipsImage,
    pub vips_source: VipsSourceCustom,
}

unsafe impl Sync for VipsImage {}
unsafe impl Send for VipsImage {}

impl Drop for VipsImage {
    fn drop(&mut self) {
        unsafe {
            if !self.vips_image.is_null() {
                libvips_sys::g_object_unref(self.vips_image as libvips_sys::gpointer);
            }
        }
    }
}

impl VipsImage {
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

pub fn new_image_from_source(source: VipsSourceCustom) -> VipsImage {
    let mut vi = VipsImage {
        vips_image: null_mut(),
        vips_source: source,
    };

    unsafe {

        let vips_image_ptr = libvips_sys::vips_image_new_from_source(
            libvips_sys::g_type_cast(
                vi.vips_source.vips_source_custom,
                libvips_sys::vips_source_get_type(),
            ),
            "\0".as_ptr() as *const c_char,
            "access\0".as_ptr() as *const c_char,
            libvips_sys::VipsAccess::VIPS_ACCESS_SEQUENTIAL,
            null_mut::<*const c_void>(),
        );

        vi.vips_image = vips_image_ptr;
    }

    vi
}
