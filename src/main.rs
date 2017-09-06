extern crate rand;
extern crate libc;
extern crate nix;
extern crate byteorder;

mod jerasurs;
mod diskio;

use rand::Rng;

fn main() {
    let codec = jerasurs::codecs::liber8tion::create(6, 64);

    let input: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(24576)
        .collect();

    let mut encoded = codec.encode(input.as_bytes());

    encoded.erase_block(0, true);
    encoded.erase_block(4, true);

    let decoded = codec.decode(&mut encoded);

    match decoded {
        Some(data) => {
            assert_eq!(data, input.as_bytes());
        },
        None => {
            println!("omg");
        }
    }
}
