extern crate libc;
extern crate rand;

mod codecs;
mod buffers;

use libc::{c_int, calloc, malloc};
use std::io::Result;
use std::mem::size_of;
use std::vec::Vec;
use codecs::Liber8tionCodec;
use codecs::Codec;
use rand::Rng;

fn main() {
    let codec = Liber8tionCodec::new();
    codec.print_bit_matrix();
    let rstr: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(500)
        .collect();

    //let input = [42u8; 15000];
    let (data, parity) = codec.encode(&rstr.as_bytes());

    let data_ptrs = data.as_ptr();

    for i in 0..data.block_count() as isize {
        unsafe {
            let block = *data_ptrs.offset(i);
            let buffer: Vec<_> = (0..data.block_size() as isize).map(|j| *block.offset(j)).collect();
            println!("{:?}", buffer);
        }
    }

    println!("parity");

    for i in 0..parity.block_count() as isize {
        unsafe {
            let block = *data_ptrs.offset(i);
            let buffer: Vec<_> = (0..parity.block_size() as isize).map(|j| *block.offset(j)).collect();
            println!("{:?}", buffer);
        }
    }

    //let mut output = [1u8, 10];
    //let input = [0u8, 10];

    //codec.encode(&input, &mut output);
}
