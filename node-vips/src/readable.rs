#![deny(clippy::all)]

use parking_lot::Mutex;
use std::sync::Arc;

use libvips_rs::VipsImage;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode},
    CallContext, JsBuffer, JsBufferValue, JsFunction, JsNumber, JsObject, JsUndefined, JsUnknown,
    Ref, Result, ValueType,
};

#[js_function(5)]
pub fn create_vips_image(ctx: CallContext) -> Result<JsUndefined> {
    let resolve_func_js = ctx.get::<JsFunction>(0)?;
    let reject_func_js = ctx.get::<JsFunction>(1)?;
    let buffer_list_js = ctx.get::<JsObject>(2)?;
    let init_func_js = ctx.get::<JsFunction>(3)?;
    let resume_func_js = ctx.get::<JsFunction>(4)?;

    init_func_js.call_without_args(None).unwrap();

    let buffer_list = ctx.env.unwrap::<Arc::<crate::BufferListClass>>(&buffer_list_js)?;

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

    let resolve_tsf = ctx.env.create_threadsafe_function(
        &resolve_func_js,
        0,
        |ctx: ThreadSafeCallContext<Arc<Mutex<VipsImage>>>| {
            let mut vips_image_js = ctx.env.create_object().unwrap();
            ctx.env.wrap(&mut vips_image_js, ctx.value).unwrap();

            Ok(vec![vips_image_js])
        },
    )?;

    let resume_tsf = ctx.env.create_threadsafe_function(
        &resume_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| {
            Ok(vec![ctx.env.get_undefined().unwrap()])
        },
    )?;


    let _reject_tsf = ctx.env.create_threadsafe_function(
        &reject_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::THREAD_POOL.get().unwrap().lock();

    let buffer_list = buffer_list.clone();
    pool.execute(move || {
        let mut custom_src = libvips_rs::new_source_custom();
        custom_src.set_on_read(move |read_buf| {
            loop {
                let mut lock = buffer_list.buffer_list.lock();
                let condvar = &buffer_list.condvar;
                match lock.read(read_buf) {
                    Ok(r) => {
                        lock.gc(|buf| {
                            unref_tsf.call(Ok(buf.inner), ThreadsafeFunctionCallMode::Blocking);
                        });

                        break r as i64
                    },
                    Err(crate::buffer_list::ReadError::NeedMoreChunk) => {
                        resume_tsf.call(Ok(()), ThreadsafeFunctionCallMode::Blocking);
                        condvar.wait(&mut lock)
                    },
                    Err(crate::buffer_list::ReadError::Error) => {
                        break -1
                    }
                }
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
    let vscale = ctx.get::<JsNumber>(1)?.get_double()?;

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
