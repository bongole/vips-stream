#![deny(clippy::all)]

use std::sync::{Arc, Mutex};

use libvips_rs::VipsImage;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode},
    CallContext, Env, JsBoolean, JsExternal, JsFunction, JsNumber, JsObject, JsString, JsUndefined,
    Result, Task,
};

struct WriteVipsImageTask {
    vips_image: Arc<Mutex<VipsImage>>,
    write_tsf: ThreadsafeFunction<(Arc<WriteContextNative>, &'static [u8])>,
    vips_write_suffix: String,
}
struct WriteContextNative {
    tx: flume::Sender<Option<i64>>,
    rx: flume::Receiver<Option<i64>>,
}

impl Drop for WriteContextNative {
    fn drop(&mut self) {
        while !self.rx.is_empty() {
            if let Some(_r) = self.rx.recv().unwrap() {}
        }
    }
}

impl Task for WriteVipsImageTask {
    type Output = bool;
    type JsValue = JsBoolean;

    fn compute(&mut self) -> Result<Self::Output> {
        let (tx, rx) = flume::unbounded();
        let native = Arc::new(WriteContextNative { tx, rx });

        let mut target_custom = libvips_rs::new_target_custom();
        let write_tsf = self.write_tsf.clone();
        target_custom.set_on_write(move |write_buf| {
            unsafe {
                write_tsf.call(
                    Ok((native.clone(), std::mem::transmute(write_buf))), // expand write_buf lifetime to 'static
                    ThreadsafeFunctionCallMode::Blocking,
                );
            }

            native.rx.recv().unwrap().unwrap_or(0)
        });

        let vips_image = self.vips_image.lock().unwrap();
        Ok(vips_image.write_to_target(&target_custom, self.vips_write_suffix.as_str()))
    }

    fn resolve(self, env: Env, output: Self::Output) -> Result<Self::JsValue> {
        env.get_boolean(output)
    }
}


#[js_function(3)]
pub fn write_vips_image(ctx: CallContext) -> Result<JsObject> {
    let attached_obj = ctx.get::<JsExternal>(0)?;
    let vips_image = ctx
        .env
        .get_value_external::<Arc<Mutex<VipsImage>>>(&attached_obj)
        .unwrap()
        .clone();

    let vips_write_suffix: String = ctx.get::<JsString>(1)?.into_utf8()?.as_str()?.to_string();

    let write_func = ctx.get::<JsFunction>(2)?;
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

    let task = WriteVipsImageTask {
        vips_image,
        write_tsf,
        vips_write_suffix,
    };

    let async_task = ctx.env.spawn(task)?;

    Ok(async_task.promise_object())
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
