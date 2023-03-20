# Chisel - Decoders

[![Rust](https://github.com/jonnycoombes/chisel-decoders/actions/workflows/rust.yml/badge.svg)](https://github.com/jonnycoombes/chisel-decoders/actions/workflows/rust.yml)

## Overview

This repository contains a very simple, lean implementation of a decoder that will consume `u8` bytes from a given 
`Read` implementation, and decode into the Rust internal `char` type using UTF-8 . This is an offshoot lib from an 
ongoing toy parser project, and is used as the first stage of the scanning/lexing phase of the parser in order avoid 
unnecessary allocations during the `u8` sequence -> `char` conversion. 

Note that the implementation is pretty fast and loose, and under the covers utilises some bit-twiddlin' in conjunction 
with the *unsafe* `transmute` function to do the conversions. *No string allocations are used during conversion*. 
There is minimal checking (other than bit-masking) of the inbound bytes - it is not intended to be a full-blown UTF8 
validation library, although improved/feature-flagged validation may be added at a later date.

## Usage 

Usage is very simple, provided you have something that implements `Read` in order to source some bytes:

### Create from a slice

Just wrap your array in a reader, and then plug it into a new instance of `Utf8Decoder`:

```rust
    let buffer: &[u8] = &[0x10, 0x12, 0x23, 0x12];
    let mut reader = BufReader::new(buffer);
    let _decoder = Utf8Decoder::new(&mut reader);
```

### Create from a file 

Just crack open your file, wrap in a `Read` instance and then plug into a new instance of `Utf8Decoder`:

```rust
    let path = PathBuf::from("somefile.txt");
    let f = File::open(path);
    let mut reader = BufReader::new(f.unwrap());
    let _decoder = Utf8Decoder::new(&mut reader);
```
### Consuming `char`s

You can either pull out new `char`s from the decoder wrapped inside a `Result` type:

```rust
    loop {
        let result = decoder.decode_next();
        if result.is_err() {
            break;
        }
    }
```
Alternatively, you can just use the `Utf8Decoder` as an `Iterator`:

```rust
    let decoder = Utf8Decoder::new(&mut reader);
    for c in decoder {
        match c {
            Some(c) => ...
            None => ...
        }       
    }
```
## Building and Testing

As you would expect, just `cargo build` in order to build the crate.  In order to run tests without doc examples 
screwing things up, make sure that you run qualified tests with `cargo test --test` or similar in order to skip examples.
