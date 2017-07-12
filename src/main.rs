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

//    for i in 0..result.total_block_count() as usize {
//        println!("{:?}", result[i]);
//    }
}
