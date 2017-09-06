use std::collections::HashMap;
use std::io::{Read, Write, Cursor};
use std::mem::size_of;


use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

mod device;
mod chunk;
mod heap;


struct Bin {
//    fd: RawFd,
    top_chunk_offset: u64
}

impl Bin {
    fn allocate(size: usize) -> u64 {
        unimplemented!()
    }
}

struct Arena {
//    fd: RawFd,
    size_classes: i64,
//    bins: HashMap<usize, ChunkHeader>
}

impl Arena {

}
