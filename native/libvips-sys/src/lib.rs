#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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
