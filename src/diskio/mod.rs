use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::io::{Read, Write, Cursor};
use std::mem::size_of;

use nix::unistd::lseek64;
use nix::unistd::read;
use nix::unistd::Whence;
use nix;

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

mod device;

static CHUNK_HEADER_SIZE: usize = 3 * 8 + 1; //3 * size_of::<u64>() + size_of::<u8>();
static CHUNK_FOOTER_SIZE: usize = 8 + 1; //size_of::<u64>() + size_of::<u8>();

fn reader_at(fd: RawFd, offset: u64, size: usize) -> nix::Result<Cursor<Vec<u8>>> {
    let mut buffer = vec![0u8; size];
    lseek64(fd, offset as i64, Whence::SeekSet)
        .and_then(|_| read(fd, buffer.as_mut_slice()))
        .and_then(|_| Ok(Cursor::new(buffer)))
}

struct ChunkHeader {
    data_size: usize,
    allocated: bool,
    prev_free: u64,
    next_free: u64
}

struct ChunkFooter {
    data_size: usize,
    allocated: bool
}

impl ChunkHeader {
    fn load(reader: &mut Read) -> ChunkHeader {
        ChunkHeader {
            data_size: reader.read_u64::<NativeEndian>().unwrap() as usize,
            allocated: reader.read_u8().unwrap() == 1,
            prev_free: reader.read_u64::<NativeEndian>().unwrap(),
            next_free: reader.read_u64::<NativeEndian>().unwrap()
        }
    }

    fn dump(header: &ChunkHeader, writer: &mut Write) {
        writer.write_u64::<NativeEndian>(header.data_size as u64);
        writer.write_u8(header.allocated as u8);
        writer.write_u64::<NativeEndian>(header.prev_free as u64);
        writer.write_u64::<NativeEndian>(header.next_free as u64);
    }
}

impl ChunkFooter {
    fn load(reader: &mut Read) -> ChunkFooter {
        ChunkFooter {
            data_size: reader.read_u64::<NativeEndian>().unwrap() as usize,
            allocated: reader.read_u8().unwrap() == 1
        }
    }

    fn dump(footer: &ChunkFooter, writer: &mut Write) {
        writer.write_u64::<NativeEndian>(footer.data_size as u64);
        writer.write_u8(footer.allocated as u8);
    }
}

struct Chunk {
    fd: RawFd,
    offset: u64,
    header: ChunkHeader,
}

struct Bin {
    fd: RawFd,
    top_chunk_offset: u64
}

impl Chunk {
    fn load(fd: RawFd, offset: u64) -> Option<Chunk> {
        match reader_at(fd, offset, CHUNK_HEADER_SIZE) {
            Ok(mut reader) => Some(Chunk {
                fd: fd,
                offset: offset,
                header: ChunkHeader::load(&mut reader)
            }),
            _ => None
        }
    }

    fn allocated(&self) -> bool {
        self.header.allocated
    }

    fn offset(&self) -> u64 {
        self.offset
    }

    fn data_offset(&self) -> u64 {
        self.offset + CHUNK_HEADER_SIZE as u64
    }

    fn data_size(&self) -> usize {
        self.header.data_size
    }

    fn prev_free(&self) -> Option<Chunk> {
        match self.header.prev_free {
            0    => None,
            prev => Self::load(self.fd, prev)
        }
    }

    fn next_free(&self) -> Option<Chunk> {
        match self.header.next_free {
            0    => None,
            next => Self::load(self.fd, next)
        }
    }

    fn prev_free_in_order(&self) -> Option<Chunk> {
        let offset = self.offset - CHUNK_FOOTER_SIZE as u64;
        let result = reader_at(self.fd, offset, CHUNK_FOOTER_SIZE)
            .map(|mut reader| ChunkFooter::load(&mut reader));

        match result {
            Ok(ChunkFooter { allocated: false, data_size: size }) =>
                Chunk::load(
                    self.fd,
                    offset - size as u64 - CHUNK_HEADER_SIZE as u64
                ),
            _ => None
        }
    }

    fn next_free_in_order(&self) -> Option<Chunk> {
        let offset = self.offset +
            CHUNK_HEADER_SIZE as u64 +
            self.data_size() as u64 +
            CHUNK_FOOTER_SIZE as u64;

        Self::load(self.fd, offset).and_then(|chunk|
            if chunk.allocated() {
                Some(chunk)
            } else {
                None
            }
        )
    }
}

impl Bin {
    fn allocate(size: usize) -> u64 {
        unimplemented!()
    }
}

struct Arena {
    fd: RawFd,
    size_classes: i64,
    bins: HashMap<usize, ChunkHeader>
}

impl Arena {

}