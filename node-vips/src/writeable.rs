#![deny(clippy::all)]

use std::sync::Arc;
use parking_lot::Mutex;

use libvips_rs::VipsImage;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode},
    CallContext, JsFunction, JsNumber, JsObject, JsString, JsUndefined, Ref, Result,
};

#[js_function(1)]
pub fn drop_vips_image(ctx: CallContext) -> Result<JsUndefined> {
    let vips_image_obj = ctx.get::<JsObject>(0)?;
    ctx.env
        .drop_wrapped::<Arc<Mutex<VipsImage>>>(vips_image_obj)
        .unwrap();
    Ok(ctx.env.get_undefined().unwrap())
}

#[js_function(5)]
pub fn write_vips_image(ctx: CallContext) -> Result<JsUndefined> {
    let vips_image_obj = ctx.get::<JsObject>(0)?;
    let vips_image = ctx.env.unwrap::<Arc<Mutex<VipsImage>>>(&vips_image_obj)?;
    let vips_image_obj_ref = ctx.env.create_reference(vips_image_obj)?;

    let vips_write_suffix: String = ctx.get::<JsString>(1)?.into_utf8()?.as_str()?.to_string();

    let resolve_func_js = ctx.get::<JsFunction>(2)?;
    let reject_func_js = ctx.get::<JsFunction>(3)?;

    let write_func_js = ctx.get::<JsFunction>(4)?;
    let write_tsf = ctx.env.create_threadsafe_function(
        &write_func_js,
        0,
        |ctx: ThreadSafeCallContext<(flume::Sender<Option<i64>>, &[u8])>| {
            let mut tx_js = ctx.env.create_object()?;
            ctx.env.wrap(&mut tx_js, ctx.value.0)?;
            let buffer_js = ctx.env.create_buffer_copy(ctx.value.1).unwrap().into_raw();

            Ok(vec![tx_js.into_unknown(), buffer_js.into_unknown()])
        },
    )?;

    let resolve_tsf = ctx.env.create_threadsafe_function(
        &resolve_func_js,
        0,
        |ctx: ThreadSafeCallContext<(Ref<()>, bool)>| {
            let vips_image_obj = ctx
                .env
                .get_reference_value_unchecked::<JsObject>(&ctx.value.0)?;
            ctx.env
                .drop_wrapped::<Arc<Mutex<VipsImage>>>(vips_image_obj)?;
            ctx.value.0.unref(ctx.env)?;

            Ok(vec![ctx.env.get_boolean(ctx.value.1).unwrap()])
        },
    )?;

    let _reject_tsf = ctx.env.create_threadsafe_function(
        &reject_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::THREAD_POOL.get().unwrap().lock();
    let (tx, rx) = flume::unbounded::<Option<i64>>();

    let vips_image = vips_image.clone();
    pool.execute(move || {
        let vips_image = vips_image.lock();

        let mut target_custom = libvips_rs::new_target_custom();
        target_custom.set_on_write(move |write_buf| {
            unsafe {
                write_tsf.call(
                    Ok((tx.clone(), std::mem::transmute(write_buf))), // expand write_buf lifetime to 'static
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }

            rx.recv().unwrap().unwrap_or(0)
        });

        let r = vips_image.write_to_target(&target_custom, vips_write_suffix.as_str());
        resolve_tsf.call(
            Ok((vips_image_obj_ref, r)),
            ThreadsafeFunctionCallMode::Blocking,
        );

        libvips_rs::clear_error();
        libvips_rs::thread_shutdown();
    });

    Ok(ctx.env.get_undefined().unwrap())
}

#[js_function(2)]
pub fn register_write_size(ctx: CallContext) -> Result<JsUndefined> {
    let tx_js = ctx.get::<JsObject>(0)?;
    let tx = ctx.env.unwrap::<flume::Sender<Option<i64>>>(&tx_js)?;
    let write_size = ctx.get::<JsNumber>(1)?.get_int64()?;

    tx.send(Some(write_size)).unwrap();

    Ok(ctx.env.get_undefined().unwrap())
}
