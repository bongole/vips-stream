#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod readable;
mod writeable;

use std::sync::Mutex;

use napi::{CallContext, JsObject, JsUndefined, Result};
use once_cell::sync::OnceCell;
use threadpool::ThreadPool;

const THREAD_POOL_SIZE:usize = 10;
static THREAD_POOL: OnceCell<Mutex<ThreadPool>> = OnceCell::new();

#[js_function(0)]
pub fn shutdown(ctx: CallContext) -> Result<JsUndefined> {
    libvips_rs::shutdown();
    ctx.env.get_undefined()
}

#[js_function(0)]
pub fn get_mem_stats(ctx: CallContext) -> Result<JsObject> {
    let mut obj = ctx.env.create_object()?;

    let tracked_mem = libvips_rs::tracked_get_mem();
    let tracked_mem_highwater = libvips_rs::tracked_get_mem_highwater();
    let cache_max_mem= libvips_rs::cache_get_max_mem();

    obj.set_named_property("mem_current", ctx.env.create_int64(tracked_mem as _)?)?;
    obj.set_named_property("mem_high", ctx.env.create_int64(tracked_mem_highwater as _)?)?;
    obj.set_named_property("mem_max", ctx.env.create_int64(cache_max_mem as _)?)?;

    let cache_size= libvips_rs::cache_get_size();
    let cache_max= libvips_rs::cache_get_max();

    obj.set_named_property("cache_current", ctx.env.create_int32(cache_size)?)?;
    obj.set_named_property("cache_max", ctx.env.create_int32(cache_max)?)?;

    Ok(obj)
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    libvips_rs::init();
    libvips_rs::leak_set(true);
    libvips_rs::cache_set_max_mem(0);
    libvips_rs::cache_set_max(0);

    let thread_pool = Mutex::new(ThreadPool::new(THREAD_POOL_SIZE));
    THREAD_POOL.set(thread_pool).unwrap();

    exports.create_named_method("createVipsImage", readable::create_vips_image)?;
    exports.create_named_method("registerReadBuf", readable::register_read_buf)?;

    exports.create_named_method("writeVipsImage", writeable::write_vips_image)?;
    exports.create_named_method("registerWriteSize", writeable::register_write_size)?;
    exports.create_named_method("dropVipsImage", writeable::drop_vips_image)?;

    exports.create_named_method("readBufTest", readable::read_buf_test)?;
    exports.create_named_method("registerReadBufTest", readable::register_read_buf_test)?;

    exports.create_named_method("shutdown", shutdown)?;
    exports.create_named_method("getMemStats", get_mem_stats)?;

    Ok(())
}
