use std::time::Instant;

use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunctionCallMode},
    CallContext, JsBuffer, JsBufferValue, JsFunction, JsNumber, JsObject, JsUndefined, JsUnknown,
    Ref, Result, ValueType,
};

#[js_function(1)]
pub fn call_test(ctx: CallContext) -> Result<JsUndefined> {
    let func_js = ctx.get::<JsFunction>(0)?;

    let func_tsf = ctx.env.create_threadsafe_function(
        &func_js,
        0,
        |ctx: ThreadSafeCallContext<flume::Sender<u64>>| {
            let mut tx_js = ctx.env.create_object()?;
            ctx.env.wrap(&mut tx_js, ctx.value)?;

            Ok(vec![tx_js.into_unknown()])
        },
    )?;

    let unref_func_js = ctx
        .env
        .create_function_from_closure("_unref_func", |ctx| ctx.env.get_undefined())
        .unwrap();
    let unref_tsf = ctx.env.create_threadsafe_function(
        &unref_func_js,
        0,
        |ctx: ThreadSafeCallContext<u64>| Ok(vec![ctx.env.get_undefined().unwrap()]),
    )?;

    let pool = crate::THREAD_POOL.get().unwrap().lock();
    let (tx, rx) = flume::unbounded::<u64>();

    pool.execute(move || loop {
        let start = Instant::now();
        func_tsf.call(Ok(tx.clone()), ThreadsafeFunctionCallMode::Blocking);
        let _r = rx.recv().unwrap();
        unref_tsf.call(Ok(0), ThreadsafeFunctionCallMode::Blocking);
        //println!("call elapsed {:?}", start.elapsed());
        let _ = start.elapsed();
    });

    ctx.env.get_undefined()
}

#[js_function(1)]
pub fn register_call_test(ctx: CallContext) -> Result<JsUndefined> {
    let tx_js = ctx.get::<JsObject>(0)?;
    let tx = ctx.env.unwrap::<flume::Sender<u64>>(&tx_js)?;

    tx.send(1).unwrap();

    ctx.env.get_undefined()
}

#[js_function(2)]
pub fn read_buf_test(ctx: CallContext) -> Result<JsUndefined> {
    let read_size = ctx.get::<JsNumber>(0)?.get_int64()?;
    let read_func_js = ctx.get::<JsFunction>(1)?;

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

    let pool = crate::THREAD_POOL.get().unwrap().lock();
    let (tx, rx) = flume::unbounded::<Option<Ref<JsBufferValue>>>();

    pool.execute(move || loop {
        //let start = Instant::now();
        read_tsf.call(Ok((tx.clone(), read_size)), ThreadsafeFunctionCallMode::Blocking);

        let r = rx.recv().unwrap();
        match r {
            Some(buf) => {
                //let buf_len = buf.len();
                unref_tsf.call(Ok(buf), ThreadsafeFunctionCallMode::Blocking);
                //println!("elapsed {:?} buf_len {:?}", start.elapsed(), buf_len);
            },
            None => break
        }
    });

    ctx.env.get_undefined()
}

#[js_function(2)]
pub fn register_read_buf_test(ctx: CallContext) -> Result<JsUndefined> {
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