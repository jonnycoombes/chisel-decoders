use std::env;
use std::fs::File;
use std::io::BufReader;
use chisel_decoders::utf8::Utf8Decoder;

fn fuzz_file() -> File {
    let path = env::current_dir()
        .unwrap()
        .join("tests/fixtures/fuzz.txt");
    File::open(path).unwrap()
}

#[test]
fn can_create_from_array() {
    let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
    let reader = BufReader::new(buffer);
    let _decoder = Utf8Decoder::new(reader);
}

#[test]
fn can_create_from_file() {
    let reader = BufReader::new(fuzz_file());
    let _decoder = Utf8Decoder::new(reader);
}

#[test]
fn pass_a_fuzz_test() {
    let reader = BufReader::new(fuzz_file());
    let mut decoder = Utf8Decoder::new(reader);
    let mut count = 0;
    while decoder.decode_next().is_ok() { count+= 1 }
    assert_eq!(count, 35283)
}

#[test]
fn should_be_an_iterator() {
    let reader = BufReader::new( fuzz_file());
    let decoder = Utf8Decoder::new(reader);
    assert_eq!(decoder.count(), 35283);
}
