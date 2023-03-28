use std::fs::File;
use std::io::BufReader;
use criterion::{criterion_group, criterion_main, Criterion};
use chisel_decoders::utf8::Utf8Decoder;

macro_rules! build_decode_benchmark {
    ($func : tt, $filename : expr) => {
        fn $func() {
            let f = File::open(format!("fixtures/{}", $filename)).unwrap();
            let reader = BufReader::new(f);
            let decoder = Utf8Decoder::new(reader);
            let mut _count = 0;
            while decoder.decode_next().is_ok() { _count+= 1}
        }
    };
}

build_decode_benchmark!(fuzz, "fuzz.txt");
build_decode_benchmark!(twitter, "twitter.json");



fn benchmark_fuzz(c: &mut Criterion) {
    c.bench_function("decode utf-8 fuzz file", |b| b.iter(fuzz));
}
fn benchmark_twitter(c: &mut Criterion) {
    c.bench_function("decode sample twitter json", |b| b.iter(twitter));
}

criterion_group!(benches, benchmark_twitter, benchmark_fuzz);
criterion_main!(benches);
