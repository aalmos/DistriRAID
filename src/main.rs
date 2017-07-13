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
    
    let mut encoded = codec.encode(input.as_bytes());
    let damage =vec![0u8; encoded.block_size()];

    encoded.overwrite_block(0, damage.as_slice());
    encoded.mark_absent(0);
    encoded.overwrite_block(4, damage.as_slice());
    encoded.mark_absent(4);

    let decoded = codec.decode(&mut encoded);

    match decoded {
        Some(data) => {
            assert_eq!(data, input.as_bytes());
        },
        None => {
            println!("fuck");
        }
    }
}
