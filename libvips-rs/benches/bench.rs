use criterion::{criterion_group, criterion_main, Criterion, black_box};
use std::{time::Instant};
use std::{
    fs::File,
    io::{BufReader, Read, Write},
};

fn resize(width: i32) {
    libvips_rs::init();
    //libvips_rs::leak_set(true);

    libvips_rs::cache_set_max_mem(0);
    libvips_rs::cache_set_max(0);

    let mut src = libvips_rs::new_source_custom();
    let mut target = libvips_rs::new_target_custom();

    let file_path = format!("{}/tests/assets/4k.jpg", env!("CARGO_MANIFEST_DIR"));
    let mut infile = File::open(file_path).unwrap();

    src.set_on_read(move |buf| infile.read(buf).unwrap() as i64);

    let mut outfile = File::create("/dev/null").unwrap();
    target.set_on_write(move |buf| outfile.write(buf).unwrap() as i64);

    let vi = libvips_rs::thumbnail_from_source(src, width).unwrap();
    vi.write_to_target(&target, ".jpg");

    libvips_rs::clear_error();
    libvips_rs::thread_shutdown();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut g = c.benchmark_group("mygroup");
    g.sample_size(30);
    g.bench_function("resize", |b| b.iter(|| resize(black_box(1000))));
    g.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
