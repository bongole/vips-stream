#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod readable;
mod writeable;

use napi::{JsObject, Result};

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    libvips_rs::init();

    exports.create_named_method("createVipsImage", readable::create_vips_image)?;
    exports.create_named_method("registerReadBuf", readable::register_read_buf)?;
    exports.create_named_method("registerReadEnd", readable::register_read_end)?;
    exports.create_named_method("writeVipsImage", writeable::write_vips_image)?;
    exports.create_named_method("registerWriteSize", writeable::register_write_size)?;

    Ok(())
}
