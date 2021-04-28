use criterion::{criterion_group, criterion_main, Criterion, black_box};
use std::{time::Instant};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
};

fn resize(thread_size: u32) {
    libvips_rs::init();
    //libvips_rs::leak_set(true);

    //libvips_rs::cache_set_max_mem(0);
    //libvips_rs::cache_set_max(0);

    let mut ts = vec![];
    for _ in 0..thread_size {
        let t = std::thread::spawn(|| {
            let mut src = libvips_rs::new_source_custom();
            let mut target = libvips_rs::new_target_custom();

            let file_path = format!("{}/tests/assets/4k.jpg", env!("CARGO_MANIFEST_DIR"));
            //let mut infile = BufReader::new(File::open(file_path).unwrap());
            let mut infile = File::open(file_path).unwrap();

            src.set_on_read(move |buf| infile.read(buf).unwrap() as i64);

            let mut outfile = File::create("/dev/null").unwrap();
            target.set_on_write(move |buf| outfile.write(buf).unwrap() as i64);

            //let start = Instant::now();
            let vi = libvips_rs::thumbnail_from_source(src, 300);
            vi.write_to_target(&target, ".jpg");
            // let duration = start.elapsed();
            // println!("elapsed {:?}", duration);
        });
        ts.push(t);
    }

    for t in ts {
        t.join().unwrap();
    }

    /*
    libvips_rs::clear_error();
    libvips_rs::thread_shutdown();

    let mem = libvips_rs::tracked_get_mem() / (1024 * 1024);
    println!("mem_current {} MiB", mem);
    */
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("mygroup");
    g.sample_size(30);
    g.bench_function("resize", |b| b.iter(|| resize(black_box(30))));
    g.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
