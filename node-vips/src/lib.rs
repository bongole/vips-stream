#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

mod buffer_list;
mod flushable_buffer;
mod readable;
mod writable;

use buffer_list::BufferList;
use flushable_buffer::FlushableBuffer;
use std::{ops::Deref, os::raw::c_int};

use napi::{
    CallContext, Env, JsBoolean, JsBuffer, JsBufferValue, JsObject, JsUndefined, JsUnknown,
    Property, Ref, Result, ValueType,
};
use once_cell::sync::OnceCell;
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use threadpool::ThreadPool;

const READ_THREAD_POOL_SIZE: usize = 2;
static READ_THREAD_POOL: OnceCell<Mutex<ThreadPool>> = OnceCell::new();
const WRITE_THREAD_POOL_SIZE: usize = 10;
static WRITE_THREAD_POOL: OnceCell<Mutex<ThreadPool>> = OnceCell::new();

#[js_function(0)]
pub fn shutdown(ctx: CallContext) -> Result<JsUndefined> {
    libvips_rs::shutdown();
    ctx.env.get_undefined()
}

#[js_function(0)]
pub fn get_mem_stats(ctx: CallContext) -> Result<JsObject> {
    let mut obj = ctx.env.create_object()?;

    let tracked_mem = libvips_rs::tracked_get_mem();
    let tracked_mem_highwater = libvips_rs::tracked_get_mem_highwater();
    let cache_max_mem = libvips_rs::cache_get_max_mem();

    obj.set_named_property("mem_current", ctx.env.create_int64(tracked_mem as _)?)?;
    obj.set_named_property(
        "mem_high",
        ctx.env.create_int64(tracked_mem_highwater as _)?,
    )?;
    obj.set_named_property("mem_max", ctx.env.create_int64(cache_max_mem as _)?)?;

    let cache_size = libvips_rs::cache_get_size();
    let cache_max = libvips_rs::cache_get_max();

    obj.set_named_property("cache_current", ctx.env.create_int32(cache_size)?)?;
    obj.set_named_property("cache_max", ctx.env.create_int32(cache_max)?)?;

    Ok(obj)
}

extern "C" {
    #[link(name = "c")]
    fn malloc_trim(__pad: usize) -> c_int;
}

#[js_function(0)]
pub fn free_memory(ctx: CallContext) -> Result<JsBoolean> {
    let r = unsafe { malloc_trim(0) };
    ctx.env.get_boolean(r == 1)
}

pub(crate) struct RefJsBufferValue {
    pub(crate) inner: Ref<JsBufferValue>,
}

impl AsRef<[u8]> for RefJsBufferValue {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl Deref for RefJsBufferValue {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.deref().deref()
    }
}

pub(crate) struct BufferListClass {
    pub(crate) buffer_list: Mutex<BufferList<RefJsBufferValue>>,
    pub(crate) condvar: Condvar,
}

#[js_function(1)]
pub fn buffer_list_class_ctor(ctx: CallContext) -> Result<JsUndefined> {
    let hwm_opt_js = ctx.get::<JsUnknown>(0)?;
    let hwm_opt_t = hwm_opt_js.get_type()?;
    let hwm: Option<usize> = if hwm_opt_t == ValueType::Null || hwm_opt_t == ValueType::Undefined {
        Some(128 * 1024) // default 128KiB
    } else {
        Some(hwm_opt_js.coerce_to_number()?.get_int64()? as usize)
    };

    let mut this: JsObject = ctx.this_unchecked();
    let native_class = BufferListClass {
        buffer_list: Mutex::new(BufferList::new(hwm)),
        condvar: Condvar::new(),
    };
    ctx.env.wrap(&mut this, Arc::new(native_class))?;

    ctx.env.get_undefined()
}

#[js_function(1)]
pub fn buffer_list_class_push(ctx: CallContext) -> Result<JsBoolean> {
    let buf_ref = ctx.get::<JsBuffer>(0)?.into_ref()?;
    let this: JsObject = ctx.this_unchecked();
    let native_class: &mut Arc<BufferListClass> = ctx.env.unwrap(&this)?;
    let mut lock = native_class.buffer_list.lock();
    let r = lock.push(RefJsBufferValue { inner: buf_ref });

    native_class.condvar.notify_one();

    ctx.env.get_boolean(r)
}

#[js_function(0)]
pub fn buffer_list_class_close(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let native_class: &mut Arc<BufferListClass> = ctx.env.unwrap(&this)?;
    let mut lock = native_class.buffer_list.lock();
    lock.close();

    native_class.condvar.notify_one();

    ctx.env.get_undefined()
}

#[js_function(1)]
pub fn flushable_buffer_class_ctor(ctx: CallContext) -> Result<JsUndefined> {
    let hwm_opt_js = ctx.get::<JsUnknown>(0)?;
    let hwm_opt_t = hwm_opt_js.get_type()?;
    let hwm: Option<usize> = if hwm_opt_t == ValueType::Null || hwm_opt_t == ValueType::Undefined {
        Some(128 * 1024) // default 128KiB
    } else {
        Some(hwm_opt_js.coerce_to_number()?.get_int64()? as usize)
    };

    let mut this: JsObject = ctx.this_unchecked();
    let native_class = Arc::new(Mutex::new(FlushableBuffer::new(hwm)));

    ctx.env.wrap(&mut this, native_class)?;

    ctx.env.get_undefined()
}

#[js_function(0)]
pub fn flushable_buffer_class_close(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let native_class: &mut Arc<Mutex<FlushableBuffer>> = ctx.env.unwrap(&this)?;
    let mut lock = native_class.lock();
    lock.close();

    ctx.env.get_undefined()
}

#[js_function(0)]
pub fn flushable_buffer_class_is_closed(ctx: CallContext) -> Result<JsBoolean> {
    let this: JsObject = ctx.this_unchecked();
    let native_class: &mut Arc<Mutex<FlushableBuffer>> = ctx.env.unwrap(&this)?;
    let lock = native_class.lock();

    ctx.env.get_boolean(lock.is_closed())
}

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    libvips_rs::init();
    // libvips_rs::leak_set(true);
    libvips_rs::set_concurrency(1);
    libvips_rs::cache_set_max_mem(0);
    libvips_rs::cache_set_max(0);
    libvips_rs::cache_set_max_files(0);
    // libvips_rs::cache_set_max_mem(50 * 1024 * 1024);
    // libvips_rs::cache_set_max(100);

    let read_thread_pool = Mutex::new(ThreadPool::new(READ_THREAD_POOL_SIZE));
    READ_THREAD_POOL.set(read_thread_pool).unwrap();
    let write_thread_pool = Mutex::new(ThreadPool::new(WRITE_THREAD_POOL_SIZE));
    WRITE_THREAD_POOL.set(write_thread_pool).unwrap();

    exports.create_named_method("createVipsImage", readable::create_vips_image)?;
    exports.create_named_method("thumbnail", readable::vips_image_thumbnail)?;
    exports.create_named_method("resize", readable::vips_image_resize)?;

    exports.create_named_method("writeVipsImage", writable::write_vips_image)?;

    exports.create_named_method("shutdown", shutdown)?;
    exports.create_named_method("getMemStats", get_mem_stats)?;
    exports.create_named_method("freeMemory", free_memory)?;

    let buffer_list_class = env.define_class(
        "BufferList",
        buffer_list_class_ctor,
        &[
            Property::new(&env, "push")?.with_method(buffer_list_class_push),
            Property::new(&env, "close")?.with_method(buffer_list_class_close),
        ],
    )?;

    exports.set_named_property("BufferList", buffer_list_class)?;

    let flushable_buffer_class = env.define_class(
        "FlushableBuffer",
        flushable_buffer_class_ctor,
        &[
            Property::new(&env, "close")?.with_method(flushable_buffer_class_close),
            Property::new(&env, "is_closed")?.with_method(flushable_buffer_class_is_closed),
        ],
    )?;

    exports.set_named_property("FlushableBuffer", flushable_buffer_class)?;

    Ok(())
}
