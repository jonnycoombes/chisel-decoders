use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use crate::common::{DecoderErrorCode};

use crate::utf8::{Utf8Decoder};

#[test]
fn can_create_from_array() {
    let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
    let mut reader = BufReader::new(buffer);
    let _decoder = Utf8Decoder::new(&mut reader);
}

#[test]
fn can_create_from_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/utf-8/fuzz.txt");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let _decoder = Utf8Decoder::new(&mut reader);
}

#[test]
fn pass_a_fuzz_test() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/utf-8/fuzz.txt");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let mut decoder = Utf8Decoder::new(&mut reader);
    let start = Instant::now();
    while decoder.decode_next().is_ok() {}
    println!("Consumed all UTF-8 in {:?}", start.elapsed());
}

#[test]
fn process_a_simple_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/simple_structure.json");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let mut decoder = Utf8Decoder::new(&mut reader);
    let start = Instant::now();
    while decoder.decode_next().is_ok() {}
    println!("Consumed all UTF-8 in {:?}", start.elapsed());
}

#[test]
fn process_a_complex_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/twitter.json");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let mut decoder = Utf8Decoder::new(&mut reader);
    let start = Instant::now();
    while decoder.decode_next().is_ok() {}
    println!("Consumed all UTF-8 in {:?}", start.elapsed());
}

#[test]
fn process_a_large_file() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/events.json");
    let f = File::open(path);
    let mut reader = BufReader::with_capacity(4096, f.unwrap());
    let mut decoder = Utf8Decoder::new(&mut reader);
    let start = Instant::now();
    loop {
        let result = decoder.decode_next();
        if result.is_err() {
            break;
        }
    }
    println!("Consumed all UTF-8 in {:?}", start.elapsed());
}

#[test]
fn should_correctly_decode_utf8_characters() {
    let buffer: &[u8] = "ह".as_bytes();
    let mut reader = BufReader::new(buffer);
    let mut decoder = Utf8Decoder::new(&mut reader);
    let char = decoder.next().unwrap();
    assert_eq!(char, 'ह')
}

#[test]
fn should_be_an_iterator() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/events.json");
    let f = File::open(path);
    let mut reader = BufReader::new( f.unwrap());
    let decoder = Utf8Decoder::new(&mut reader);
    let start = Instant::now();
    for _c in decoder {}
    println!("Consumed all UTF-8 in {:?}", start.elapsed());
}

#[test]
fn should_produce_eof_markers() {
    let path = env::current_dir()
        .unwrap()
        .join("src/test/fixtures/samples/json/events.json");
    let f = File::open(path);
    let mut reader = BufReader::new( f.unwrap());
    let mut decoder = Utf8Decoder::new(&mut reader);
    loop {
        let result = decoder.decode_next();
        match result {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
                match err.code {
                    DecoderErrorCode::EndOfInput => {
                        break;
                    }
                    _ => {
                        panic!();
                    }
                }
            }
        }
    }
}
