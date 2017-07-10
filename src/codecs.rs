pub mod codecs;

use buffers::BlockBuffer;
use std::mem::size_of;

static LIBER8TION_W: i32 = 8;
static LIBER8TION_K: i32 = 6;
static LIBER8TION_M: i32 = 2;
static LIBER8TION_PACKET_SIZE: i32 = 8;

pub trait Codec {
    fn encode(&self, input: &BlockBuffer) -> BlockBuffer;
    fn decode(&self, input: &BlockBuffer) -> BlockBuffer;
}

struct Liber8tionCodec {
    _k: c_int,
    _w: c_int,
    _m: c_int,
    _packet_size: c_int,
    _bit_matrix: BitMatrix,
    _schedule: Schedule
}

impl Liber8tionCodec {
    fn new() -> Liber8tionCodec {
        let m = 2;
        let w = 8;
        let k = 6;
        let packet_size = size_of::<*mut u64>();

    }

}
