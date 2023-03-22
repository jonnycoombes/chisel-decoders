#![allow(dead_code)]
#![allow(clippy::transmute_int_to_char)]
//! A character-oriented decoder implementation that will take an underlying [std::u8] (byte) source
//! and produce a stream of decoded Unicode (UTF-8) characters
use std::fmt::{Debug, Formatter};
use std::io::{Bytes, Read};
use std::mem::transmute;
use crate::common::*;
use crate::decoder_error;

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

#[inline(always)]
fn decode_double(a: u32, b: u32) -> u32 {
    (b & FOLLOWING_BYTE_MASK) | ((a & DOUBLE_BYTE_MASK) << 6)
}

#[inline(always)]
fn decode_triple(a: u32, b: u32, c: u32) -> u32 {
    (c & FOLLOWING_BYTE_MASK) | ((b & FOLLOWING_BYTE_MASK) << 6) | ((a & TRIPLE_BYTE_MASK) << 12)
}

#[inline(always)]
fn decode_quad(a: u32, b: u32, c: u32, d: u32) -> u32 {
    (d & FOLLOWING_BYTE_MASK)
        | ((c & FOLLOWING_BYTE_MASK) << 6)
        | ((b & FOLLOWING_BYTE_MASK) << 12)
        | ((a & QUAD_BYTE_MASK) << 18)
}

/// A UTF-8 decoder, which is wrapped around a given [Read] instance.
/// The lifetime of the reader instance must be at least as long as the decoder
pub struct Utf8Decoder<Reader: Read + Debug> {
    /// The input stream
    input: Bytes<Reader>,
}

impl<Reader: Read + Debug> Utf8Decoder<Reader> {
    /// Create a new decoder with a default buffer size
    pub fn new(r: Reader) -> Self {
        Utf8Decoder { input: r.bytes() }
    }

    /// Attempt to decode the next character in the underlying stream. Assumes the maximum
    /// number of unicode bytes is 4 *not* 6
    pub fn decode_next(&mut self) -> DecoderResult<char> {
        let leading_byte = self.next_packed_byte()?;
        unsafe {
            if single_byte_sequence!(leading_byte) {
                return Ok(transmute(leading_byte));
            }
            if double_byte_sequence!(leading_byte) {
                return Ok(transmute(decode_double(
                    leading_byte,
                    self.next_packed_byte()?,
                )));
            }
            if triple_byte_sequence!(leading_byte) {
                return Ok(transmute(decode_triple(
                    leading_byte,
                    self.next_packed_byte()?,
                    self.next_packed_byte()?,
                )));
            }
            if quad_byte_sequence!(leading_byte) {
                return Ok(transmute(decode_quad(
                    leading_byte,
                    self.next_packed_byte()?,
                    self.next_packed_byte()?,
                    self.next_packed_byte()?,
                )));
            }
        }
        decoder_error!(
            DecoderErrorCode::InvalidByteSequence,
            "failed to decode any valid UTF-8"
        )
    }

    /// Attempt to read a single byte from the underlying stream
    #[inline(always)]
    fn next_packed_byte(&mut self) -> DecoderResult<u32> {
        match self.input.next() {
            Some(result) => match result {
                Ok(b) => Ok(b as u32),
                Err(_) => decoder_error!(DecoderErrorCode::StreamFailure, "failed to read next byte"),
            },
            None => decoder_error!(DecoderErrorCode::EndOfInput, "no more bytes available"),
        }
    }
}

impl<Reader: Read + Debug> Debug for Utf8Decoder<Reader> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "rdr: {:?}", self.input)
    }
}

impl<Reader: Read + Debug> Iterator for Utf8Decoder<Reader> {
    type Item = char;
    /// Decode the next character from the underlying stream
    fn next(&mut self) -> Option<Self::Item> {
        match self.decode_next() {
            Ok(c) => Some(c),
            Err(_) => None,
        }
    }
}
