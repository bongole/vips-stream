#![deny(clippy::all)]

use std::sync::{Arc, Mutex};

use libvips_rs::VipsImage;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode},
    CallContext, JsExternal, JsFunction, JsNumber, JsString, JsUndefined, Result,
};

struct WriteContextNative {
    tx: flume::Sender<Option<i64>>,
    rx: flume::Receiver<Option<i64>>,
}

impl Drop for WriteContextNative {
    fn drop(&mut self) {
        while !self.rx.is_empty() {
            let _ = self.rx.recv().unwrap();
        }
    }
}

#[js_function(5)]
pub fn write_vips_image(ctx: CallContext) -> Result<JsUndefined> {
    let attached_obj = ctx.get::<JsExternal>(0)?;
    let vips_image = ctx
        .env
        .get_value_external::<Arc<Mutex<VipsImage>>>(&attached_obj)
        .unwrap();

    let vips_write_suffix: String = ctx.get::<JsString>(1)?.into_utf8()?.as_str()?.to_string();

    let resolve_func_js = ctx.get::<JsFunction>(2)?;
    let reject_func_js = ctx.get::<JsFunction>(3)?;

    let write_func = ctx.get::<JsFunction>(4)?;
    let write_tsf = ctx.env.create_threadsafe_function(
        &write_func,
        0,
        |ctx: ThreadSafeCallContext<(Arc<WriteContextNative>, &[u8])>| {
            let write_ctx = ctx.env.create_external(ctx.value.0, None).unwrap();
            let buffer_js = ctx.env.create_buffer_copy(ctx.value.1).unwrap().into_raw();

            Ok(vec![
                write_ctx.coerce_to_object().unwrap(),
                buffer_js.coerce_to_object().unwrap(),
            ])
        },
    )?;

    let resolve_tsf = ctx.env.create_threadsafe_function(
        &resolve_func_js,
        0,
        |ctx: ThreadSafeCallContext<bool>| Ok(vec![ctx.env.get_boolean(ctx.value).unwrap()]),
    )?;

    let _reject_tsf = ctx.env.create_threadsafe_function(
        &reject_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::THREAD_POOL.get().unwrap().lock().unwrap();
    let (tx, rx) = flume::unbounded();
    let native = Arc::new(WriteContextNative { tx, rx });

    let vips_image = vips_image.clone();
    pool.execute(move || {
        let vips_image = vips_image.lock().unwrap();

        let mut target_custom = libvips_rs::new_target_custom();
        target_custom.set_on_write(move |write_buf| {
            unsafe {
                write_tsf.call(
                    Ok((native.clone(), std::mem::transmute(write_buf))), // expand write_buf lifetime to 'static
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }

            native.rx.recv().unwrap().unwrap_or(0)
        });

        target_custom.set_on_finish(move || {
            resolve_tsf.call(Ok(true), ThreadsafeFunctionCallMode::Blocking);
        });

        vips_image.write_to_target(&target_custom, vips_write_suffix.as_str());
    });

    Ok(ctx.env.get_undefined().unwrap())
}

#[js_function(2)]
pub fn register_write_size(ctx: CallContext) -> Result<JsUndefined> {
    let ctx_obj = ctx.get::<JsExternal>(0)?;
    let write_size = ctx.get::<JsNumber>(1)?.get_int64()?;
    let native = ctx
        .env
        .get_value_external::<Arc<WriteContextNative>>(&ctx_obj)?;

    native.tx.send(Some(write_size)).unwrap();

    Ok(ctx.env.get_undefined().unwrap())
}
