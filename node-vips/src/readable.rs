#![deny(clippy::all)]

use std::sync::Arc;
use parking_lot::Mutex;

use libvips_rs::VipsImage;
use napi::{CallContext, JsBuffer, JsBufferValue, JsFunction, JsNumber, JsObject, JsUndefined, JsUnknown, Ref, Result, ValueType, threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode}};

#[js_function(3)]
pub fn create_vips_image(ctx: CallContext) -> Result<JsUndefined> {
    let resolve_func_js = ctx.get::<JsFunction>(0)?;
    let reject_func_js = ctx.get::<JsFunction>(1)?;
    let read_func_js = ctx.get::<JsFunction>(2)?;

    let unref_func_js = ctx
        .env
        .create_function_from_closure("_unref_func", |ctx| ctx.env.get_undefined())
        .unwrap();
    let unref_tsf = ctx.env.create_threadsafe_function(
        &unref_func_js,
        0,
        |ctx: ThreadSafeCallContext<Ref<JsBufferValue>>| {
            ctx.value.unref(ctx.env).unwrap();
            Ok(vec![ctx.env.get_undefined().unwrap()])
        },
    )?;

    let read_tsf = ctx.env.create_threadsafe_function(
        &read_func_js,
        0,
        |ctx: ThreadSafeCallContext<(flume::Sender<Option<Ref<JsBufferValue>>>, i64)>| {
            let mut tx_js = ctx.env.create_object()?;
            ctx.env.wrap(&mut tx_js, ctx.value.0)?;
            let read_size_js = ctx.env.create_int64(ctx.value.1)?;

            Ok(vec![tx_js.into_unknown(), read_size_js.into_unknown()])
        },
    )?;

    let resolve_tsf = ctx.env.create_threadsafe_function(
        &resolve_func_js,
        0,
        |ctx: ThreadSafeCallContext<Arc<Mutex<VipsImage>>>| {
            let mut vips_image_js = ctx.env.create_object().unwrap();
            ctx.env.wrap(&mut vips_image_js, ctx.value).unwrap();

            Ok(vec![vips_image_js])
        },
    )?;

    let _reject_tsf = ctx.env.create_threadsafe_function(
        &reject_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::THREAD_POOL.get().unwrap().lock();
    let (tx, rx) = flume::unbounded::<Option<Ref<JsBufferValue>>>();

    pool.execute(move || {
        let mut custom_src = libvips_rs::new_source_custom();
        custom_src.set_on_read(move |read_buf| {
            read_tsf.call(
                Ok((tx.clone(), read_buf.len() as i64)),
                ThreadsafeFunctionCallMode::Blocking,
            );

            let r = rx.recv().unwrap();
            match r {
                Some(buf) => {
                    let buf_len = buf.len();
                    read_buf[..buf_len].copy_from_slice(buf.as_ref());
                    unref_tsf.call(Ok(buf), ThreadsafeFunctionCallMode::Blocking);
                    buf_len as i64
                }
                None => 0,
            }
        });

        //let vi = libvips_rs::new_image_from_source(custom_src);
        let vi = libvips_rs::thumbnail_from_source(custom_src, 300);
        resolve_tsf.call(
            Ok(Arc::new(Mutex::new(vi))),
            ThreadsafeFunctionCallMode::Blocking,
        );

        libvips_rs::clear_error();
        libvips_rs::thread_shutdown();
    });

    ctx.env.get_undefined()
}

#[js_function(2)]
pub fn vips_image_thumbnail(ctx: CallContext) -> Result<JsUndefined> {
    let vips_image_obj = ctx.get::<JsObject>(0)?;
    let vips_image = ctx.env.unwrap::<Arc<Mutex<VipsImage>>>(&vips_image_obj)?;
    let width = ctx.get::<JsNumber>(1)?.get_int32()?;

    vips_image.lock().thumbnail(width);

    ctx.env.get_undefined()
}

#[js_function(2)]
pub fn vips_image_resize(ctx: CallContext) -> Result<JsUndefined> {
    let vips_image_obj = ctx.get::<JsObject>(0)?;
    let vips_image = ctx.env.unwrap::<Arc<Mutex<VipsImage>>>(&vips_image_obj)?;
    let vscale= ctx.get::<JsNumber>(1)?.get_double()?;

    vips_image.lock().resize(vscale);

    ctx.env.get_undefined()
}


#[js_function(2)]
pub fn register_read_buf(ctx: CallContext) -> Result<JsUndefined> {
    let tx_js = ctx.get::<JsObject>(0)?;
    let tx = ctx
        .env
        .unwrap::<flume::Sender<Option<Ref<JsBufferValue>>>>(&tx_js)?;
    let js_unknown = ctx.get::<JsUnknown>(1)?;
    let t = js_unknown.get_type()?;

    if t == ValueType::Null || t == ValueType::Undefined {
        tx.send(None).unwrap()
    } else {
        let js_buffer_ref = unsafe { js_unknown.cast::<JsBuffer>() }.into_ref()?;
        tx.send(Some(js_buffer_ref)).unwrap()
    }

    ctx.env.get_undefined()
}

