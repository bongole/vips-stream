#![deny(clippy::all)]

use parking_lot::Mutex;
use std::sync::Arc;

use libvips_rs::VipsImage;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode},
    CallContext, JsFunction, JsObject, JsString, JsUndefined, Ref, Result,
};

use crate::flushable_buffer::FlushableBuffer;

#[js_function(5)]
pub fn write_vips_image(ctx: CallContext) -> Result<JsUndefined> {
    let vips_image_obj = ctx.get::<JsObject>(0)?;
    let vips_image = ctx.env.unwrap::<Arc<Mutex<VipsImage>>>(&vips_image_obj)?;
    let vips_image_obj_ref = ctx.env.create_reference(vips_image_obj)?;

    let vips_write_suffix: String = ctx.get::<JsString>(1)?.into_utf8()?.as_str()?.to_string();
    let flushable_buffer_obj = ctx.get::<JsObject>(2)?;
    let flushable_buffer = ctx.env.unwrap::<Arc<Mutex<FlushableBuffer>>>(&flushable_buffer_obj)?;

    let reject_func_js = ctx.get::<JsFunction>(3)?;

    let write_func_js = ctx.get::<JsFunction>(4)?;
    let write_tsf = ctx.env.create_threadsafe_function(
        &write_func_js,
        1,
        |ctx: ThreadSafeCallContext<(Box<[u8]>, bool)>| {
            let buffer_js = ctx.env.create_buffer_copy(ctx.value.0).unwrap().into_raw();
            let end_js = ctx.env.get_boolean(ctx.value.1)?;

            Ok(vec![buffer_js.into_unknown(), end_js.into_unknown()])
        },
    )?;

    let unref_func_js = ctx
        .env
        .create_function_from_closure("__unref", |ctx| ctx.env.get_undefined())?;

    let unref_func_tsf = ctx.env.create_threadsafe_function(
        &unref_func_js,
        0,
        |ctx: ThreadSafeCallContext<Ref<()>>| {
            let vips_image_obj = ctx
                .env
                .get_reference_value_unchecked::<JsObject>(&ctx.value)?;
            ctx.env
                .drop_wrapped::<Arc<Mutex<VipsImage>>>(vips_image_obj)?;
            ctx.value.unref(ctx.env)?;

            Ok(vec![ctx.env.get_undefined().unwrap()])
        },
    )?;

    let reject_tsf = ctx.env.create_threadsafe_function(
        &reject_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::WRITE_THREAD_POOL.get().unwrap().lock();

    let vips_image = vips_image.clone();
    let fb = flushable_buffer.clone();
    pool.execute(move || {
        let vips_image = vips_image.lock();

        let mut target_custom = libvips_rs::new_target_custom();

        let fb_clone = fb.clone();
        let write_tsf_clone = write_tsf.clone();
        target_custom.set_on_write(move |write_buf| {
            let mut fb = fb_clone.lock();
            if fb.is_closed() {
                return -1;
            }

            if !fb.write(write_buf) {
                fb.flush(|b| {
                    write_tsf_clone.call(
                        Ok((Box::from(b), false)),
                        ThreadsafeFunctionCallMode::Blocking,
                    );
                });
            }

            write_buf.len() as i64
        });

        let fb_clone = fb.clone();
        target_custom.set_on_finish(move || {
            let mut fb = fb_clone.lock();
            fb.close();
            fb.flush(|b| {
                write_tsf.call(
                    Ok((Box::from(b), true)),
                    ThreadsafeFunctionCallMode::Blocking,
                );
            });
        });

        let r = vips_image.write_to_target(&target_custom, vips_write_suffix.as_str());
        unref_func_tsf.call(Ok(vips_image_obj_ref), ThreadsafeFunctionCallMode::Blocking);
        if !r {
            reject_tsf.call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
        }

        libvips_rs::clear_error();
        libvips_rs::thread_shutdown();
    });

    Ok(ctx.env.get_undefined().unwrap())
}
