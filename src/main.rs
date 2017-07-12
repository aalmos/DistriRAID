extern crate rand;
extern crate libc;

mod jerasurs;

use rand::Rng;

fn main() {
    let codec = jerasurs::codecs::liber8tion::create(6, 64);
    
    codec.print_bit_matrix();
    
    let input: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(24576)
        .collect();
    
    let result = codec.encode(input.as_bytes());

    for i in 0..result.total_block_count() as usize {
        println!("{:?}", result[i]);
    }
/*
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
*/
}
