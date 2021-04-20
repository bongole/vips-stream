#[cfg(test)]
mod bench_tests {

    use std::{
        fs::File,
        io::{Read, Write},
    };

    #[test]
    fn bench() {
        libvips_rs::init();
        libvips_rs::leak_set(true);

        libvips_rs::cache_set_max_mem(0);
        libvips_rs::cache_set_max(0);

        for _ in 0..100 {
            let mut src = libvips_rs::new_source_custom();
            let mut target = libvips_rs::new_target_custom();

            let file_path = format!("{}/tests/assets/4k.jpg", env!("CARGO_MANIFEST_DIR"));
            let mut file = File::open(file_path).unwrap();

            src.set_on_read(move |buf| file.read(buf).unwrap() as i64);

            let mut tmpfile = File::create("/dev/null").unwrap();
            target.set_on_write(move |buf| tmpfile.write(buf).unwrap() as i64);
            target.set_on_finish(|| println!("on_finish"));

            let vi = libvips_rs::new_image_from_source(src);
            let vi = vi.thumbnail(300);
            vi.write_to_target(&target, ".png");

            libvips_rs::clear_error();
            libvips_rs::thread_shutdown();

            let mem = libvips_rs::tracked_get_mem() / (1024 * 1024);
            println!("mem_current {} MiB", mem);

        }

        unsafe { libvips_sys::vips_object_print_all() }

    }
}
