use std::fs::File;
use std::io::BufReader;
use criterion::{criterion_group, criterion_main, Criterion};
use chisel_decoders::utf8::Utf8Decoder;

fn decode() {
    let f = File::open("fixtures/twitter.json").unwrap();
    let reader = BufReader::new(f);
    let decoder = Utf8Decoder::new(reader);
    let mut _count = 0;
    while decoder.decode_next().is_ok() { _count += 1; }
}

fn fuzz_benchmark(c: &mut Criterion) {
    c.bench_function("decode twitter extract", |b| b.iter(decode));
}

criterion_group!(benches, fuzz_benchmark);
criterion_main!(benches);
