#![deny(clippy::all)]

use std::sync::{Arc, Mutex};

use libvips_rs::VipsImage;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    CallContext, JsBuffer, JsBufferValue, JsExternal, JsFunction, JsUndefined, Ref, Result,
};

struct ReadContextNative {
    tx: flume::Sender<Option<Ref<JsBufferValue>>>,
    rx: flume::Receiver<Option<Ref<JsBufferValue>>>,
    unref_tsf: ThreadsafeFunction<Ref<JsBufferValue>>,
}

impl Drop for ReadContextNative {
    fn drop(&mut self) {
        while !self.rx.is_empty() {
            if let Some(r) = self.rx.recv().unwrap() {
                self.unref_tsf
                    .call(Ok(r), ThreadsafeFunctionCallMode::Blocking);
            }
        }
    }
}

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
        |ctx: ThreadSafeCallContext<(Arc<ReadContextNative>, i64)>| {
            let ctx_js = ctx.env.create_external(ctx.value.0, None)?;
            let read_size_js = ctx.env.create_int64(ctx.value.1)?;

            Ok(vec![
                ctx_js.coerce_to_object().unwrap(),
                read_size_js.coerce_to_object().unwrap(),
            ])
        },
    )?;

    let resolve_tsf = ctx.env.create_threadsafe_function(
        &resolve_func_js,
        0,
        |ctx: ThreadSafeCallContext<Arc<Mutex<VipsImage>>>| {
            let vips_image_js = ctx.env.create_external(ctx.value, None)?;

            Ok(vec![vips_image_js])
        },
    )?;

    let _reject_tsf = ctx.env.create_threadsafe_function(
        &reject_func_js,
        0,
        |ctx: ThreadSafeCallContext<()>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::THREAD_POOL.get().unwrap().lock().unwrap();
    let (tx, rx) = flume::unbounded();
    let native = Arc::new(ReadContextNative { tx, rx, unref_tsf });

    pool.execute(move || {
        let mut custom_src = libvips_rs::new_source_custom();
        custom_src.set_on_read(move |read_buf| {
            read_tsf.call(
                Ok((native.clone(), read_buf.len() as i64)),
                ThreadsafeFunctionCallMode::Blocking,
            );

            let r = native.rx.recv().unwrap();
            match r {
                Some(buf) => {
                    let buf_len = buf.len();
                    read_buf[..buf_len].copy_from_slice(buf.as_ref());
                    native
                        .unref_tsf
                        .call(Ok(buf), ThreadsafeFunctionCallMode::Blocking);
                    buf_len as i64
                }
                None => 0,
            }
        });

        let vi = libvips_rs::new_image_from_source(custom_src);
        let vi = vi.thumbnail(300); // TODO image processing
        resolve_tsf.call(
            Ok(Arc::new(Mutex::new(vi))),
            ThreadsafeFunctionCallMode::Blocking,
        );
    });

    Ok(ctx.env.get_undefined().unwrap())
}

#[js_function(2)]
pub fn register_read_buf(ctx: CallContext) -> Result<JsUndefined> {
    let ctx_obj = ctx.get::<JsExternal>(0)?;
    let js_buffer_ref = ctx.get::<JsBuffer>(1)?.into_ref().unwrap();
    let native = ctx
        .env
        .get_value_external::<Arc<ReadContextNative>>(&ctx_obj)?;

    native.tx.send(Some(js_buffer_ref)).unwrap();

    ctx.env.get_undefined()
}

#[js_function(1)]
pub fn register_read_end(ctx: CallContext) -> Result<JsUndefined> {
    let ctx_obj = ctx.get::<JsExternal>(0)?;
    let native = ctx
        .env
        .get_value_external::<Arc<ReadContextNative>>(&ctx_obj)?;

    native.tx.send(None).unwrap();

    ctx.env.get_undefined()
}
