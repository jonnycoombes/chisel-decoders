#![allow(dead_code)]
#![allow(clippy::transmute_int_to_char)]
//! A character-oriented decoder implementation that will take an underlying [std::u8] (byte) source
//! and produce a stream of decoded Unicode (UTF-8) characters
use std::cell::RefCell;
use std::io::BufRead;
use std::mem::transmute;

use crate::{end_of_input, invalid_byte_sequence};
use crate::common::*;
use crate::utf8::SequenceType::Unrecognised;

enum SequenceType {
    Single,
    Pair,
    Triple,
    Quad,
    Unrecognised,
}

/// Mask for extracting 7 bits from a single byte sequence
const SINGLE_BYTE_MASK: u32 = 0b0111_1111;
/// Mask for extracting initial 5 bits within a double byte UTF-8 sequence
const DOUBLE_BYTE_MASK: u32 = 0b0001_1111;
/// Mask for extracting initial 4 bits within a triple byte UTF-8 ssequence
const TRIPLE_BYTE_MASK: u32 = 0b0000_1111;
/// Mask for extracting initial 3 bits within a quad byte UTF-8 ssequence
const QUAD_BYTE_MASK: u32 = 0b0000_0111;
/// Mask for extracting 6 bits from following byte UTF-8 ssequences
const FOLLOWING_BYTE_MASK: u32 = 0b0011_1111;

/// Convenience macro for some bit twiddlin'
macro_rules! single_byte_sequence {
    ($byte : expr) => {
        $byte >> 7 == 0b0000_0000
    };
}

/// Convenience macro for some bit twiddlin'
macro_rules! double_byte_sequence {
    ($byte : expr) => {
        $byte >> 5 == 0b0000_0110
    };
}

/// Convenience macro for some bit twiddlin'
macro_rules! triple_byte_sequence {
    ($byte : expr) => {
        $byte >> 4 == 0b0000_1110
    };
}

/// Convenience macro for some bit twiddlin'
macro_rules! quad_byte_sequence {
    ($byte : expr) => {
        $byte >> 3 == 0b0001_1110
    };
}

macro_rules! decode_pair {
    ($buf : expr) => {
        ($buf[1] as u32 & FOLLOWING_BYTE_MASK)
        | (($buf[0] as u32 & DOUBLE_BYTE_MASK) << 6)
    }
}

macro_rules! decode_triple {
    ($buf : expr) => {
        ($buf[2] as u32 & FOLLOWING_BYTE_MASK)
        | (($buf[1] as u32 & FOLLOWING_BYTE_MASK) << 6)
        | (($buf[0] as u32 & TRIPLE_BYTE_MASK) << 12)
    }
}

macro_rules! decode_quad {
    ($buf : expr) => {
        ($buf[3] as u32 & FOLLOWING_BYTE_MASK)
        | (($buf[2] as u32 & FOLLOWING_BYTE_MASK) << 6)
        | (($buf[1] as u32 & FOLLOWING_BYTE_MASK) << 12)
        | (($buf[0] as u32 & QUAD_BYTE_MASK) << 18)
    }
}

/// Determine what kind of UTF-8 sequence we're dealing with
#[inline(always)]
fn sequence_type(b: u8) -> SequenceType {
    if single_byte_sequence!(b) {
        SequenceType::Single
    } else if double_byte_sequence!(b) {
        SequenceType::Pair
    } else if triple_byte_sequence!(b) {
        SequenceType::Triple
    } else if quad_byte_sequence!(b) {
        SequenceType::Quad
    } else {
        Unrecognised
    }
}

/// A UTF-8 decoder, which is wrapped around a given [Read] instance.
/// The lifetime of the reader instance must be at least as long as the decoder
pub struct Utf8Decoder<B: BufRead> {
    /// The input stream
    input: RefCell<B>,
}

impl<B: BufRead> Utf8Decoder<B> {
    /// Create a new decoder with a default buffer size
    pub fn new(r: B) -> Self {
        Utf8Decoder { input: RefCell::new(r) }
    }

    /// Attempt to decode the next character in the underlying stream. Assumes the maximum
    /// number of unicode bytes is 4 *not* 6
    pub fn decode_next(&self) -> DecoderResult<char> {
        let mut buffer: [u8; 4] = [0; 4];
        let mut input = self.input.borrow_mut();
        if let Ok(count) = input.read(&mut buffer[0..1]) {
            return if count != 1 {
                end_of_input!()
            } else {
                match sequence_type(buffer[0]) {
                    SequenceType::Single => {
                        unsafe { Ok(transmute(buffer[0] as u32)) }
                    }
                    SequenceType::Pair => {
                        input.read_exact(&mut buffer[1..2])
                            .expect("failed to read sequence suffix");
                        unsafe {
                            Ok(transmute(decode_pair!(&buffer[0..2])))
                        }
                    }
                    SequenceType::Triple => {
                        input.read_exact(&mut buffer[1..3])
                            .expect("failed to read sequence suffix");
                        unsafe {
                            Ok(transmute(decode_triple!(&buffer[0..3])))
                        }
                    }
                    SequenceType::Quad => {
                        input.read_exact(&mut buffer[1..4])
                            .expect("failed to read sequence suffix");
                        unsafe {
                            Ok(transmute(decode_quad!(&buffer[0..4])))
                        }
                    }
                    Unrecognised => {
                        invalid_byte_sequence!()
                    }
                }
            };
        }
        Ok('c')
    }
}

impl<B: BufRead> Iterator for Utf8Decoder<B> {
    type Item = char;
    /// Decode the next character from the underlying stream
    fn next(&mut self) -> Option<Self::Item> {
        match self.decode_next() {
            Ok(c) => Some(c),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::time::Instant;

    use crate::utf8::Utf8Decoder;

    fn fuzz_file() -> File {
        File::open("fixtures/fuzz.txt").unwrap()
    }

    fn complex_file() -> File { File::open("fixtures/twitter.json").unwrap() }

    #[test]
    fn can_create_from_array() {
        let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
        let reader = BufReader::new(buffer);
        let decoder = Utf8Decoder::new(reader);
        let mut _count = 0;
        while decoder.decode_next().is_ok() { _count += 1; }
    }

    #[test]
    fn can_create_from_file() {
        let reader = BufReader::new(fuzz_file());
        let _decoder = Utf8Decoder::new(reader);
    }

    #[test]
    fn pass_a_fuzz_test() {
        let start = Instant::now();
        let reader = BufReader::new(fuzz_file());
        let decoder = Utf8Decoder::new(reader);
        let mut count = 0;
        while decoder.decode_next().is_ok() { count += 1; }
        assert_eq!(count, 35283);
        println!("Decoded fuzz file in {:?}", start.elapsed());
    }

    #[test]
    fn decode_a_complex_document() {
        let reader = BufReader::new(complex_file());
        let decoder = Utf8Decoder::new(reader);
        let mut count = 0;
        while decoder.decode_next().is_ok() { count += 1; }
        assert_eq!(count, 567916);
    }

    #[test]
    fn should_be_an_iterator() {
        let start = Instant::now();
        let reader = BufReader::new(fuzz_file());
        let decoder = Utf8Decoder::new(reader);
        assert_eq!(decoder.count(), 35283);
        println!("Counted fuzz file in {:?}", start.elapsed());
    }
}
