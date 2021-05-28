#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[inline]
pub unsafe fn g_signal_connect(
    instance: gpointer,
    detailed_signal: *const gchar,
    c_handler: GCallback,
    data: gpointer,
) -> gulong {
    g_signal_connect_data(
        instance,
        detailed_signal,
        c_handler,
        data,
        None,
        std::mem::transmute(0),
    )
}

#[inline]
pub unsafe fn g_type_cast<T, U>(instance: *mut T, iface_type: GType) -> *mut U {
    g_type_check_instance_cast(instance as *mut GTypeInstance, iface_type) as *mut U
}