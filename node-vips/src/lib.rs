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

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    libvips_rs::init();
    //libvips_rs::leak_set(true);

    let thread_pool = Mutex::new(ThreadPool::new(THREAD_POOL_SIZE));
    THREAD_POOL.set(thread_pool).unwrap();

    exports.create_named_method("createVipsImage", readable::create_vips_image)?;
    exports.create_named_method("registerReadBuf", readable::register_read_buf)?;
    exports.create_named_method("registerReadEnd", readable::register_read_end)?;

    exports.create_named_method("writeVipsImage", writeable::write_vips_image)?;
    exports.create_named_method("registerWriteSize", writeable::register_write_size)?;
    exports.create_named_method("dropVipsImage", writeable::drop_vips_image)?;

    exports.create_named_method("shutdown", shutdown)?;

    Ok(())
}
