use std::sync::{Arc, Mutex};
use neon::prelude::*;

fn buffer_check(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let buf = cx.argument::<JsBuffer>(0)?;
    let slice = {
        let guard = cx.lock();
        let data = buf.borrow(&guard);
        data.as_slice::<u8>()
    };

    println!("{:?}", slice);

    Ok(cx.undefined())
}

#[allow(clippy::unnecessary_wraps)]
fn thread_test(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    let queue = cx.queue();

    println!("a {:?}", std::thread::current());

    std::thread::spawn(move || {
        loop {
            println!("b {:?}", std::thread::current());
            queue.send(move |mut _cx| {
                println!("c {:?}", std::thread::current());
                Ok(())
            });

            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    Ok(cx.undefined())
}

pub struct Vips {
    vips_image: libvips_rs::VipsImage,
}

impl Finalize for Vips {}

#[allow(clippy::unnecessary_wraps)]
fn vips_new(mut cx: FunctionContext) -> JsResult<JsBox<Arc<Mutex<Vips>>>> {
    let mut src = libvips_rs::new_source_custom();
    src.set_on_read(move |_buf|{
        println!("read");
        0
    });

    let img = libvips_rs::new_image_from_source(src);
    let vips = Arc::new(Mutex::new(Vips{ vips_image: img }));

    println!("vips_new {:?}", std::thread::current());

    /*
    let vips_clone = vips.clone();
    std::thread::spawn(move || {
        println!("vips_spawn {:?}", std::thread::current());
        vips_clone.lock().unwrap().vips_image.vips_source.set_on_read(move |buf| {
            println!("set_on_read {:?}", std::thread::current());
            0
        });
        println!("vips_spawn after {:?}", std::thread::current());
    });
    */

    Ok(cx.boxed(vips))
}


#[neon::main]
fn module_main(mut m: ModuleContext) -> NeonResult<()> {
    libvips_rs::init();

    m.export_function("buffer_check", buffer_check)?;
    m.export_function("thread_test", thread_test)?;
    m.export_function("vips_new", vips_new)?;

    Ok(())
}
