use std::io::{Read, Write, Cursor};
use std::os::unix::io::RawFd;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

static CHUNK_HEADER_SIZE: usize = 3 * 8 + 1; //3 * size_of::<u64>() + size_of::<u8>();
static CHUNK_FOOTER_SIZE: usize = 8 + 1; //size_of::<u64>() + size_of::<u8>();

struct ChunkHeader {
    data_size: usize,
    allocated: bool,
    prev_free: u64,
    next_free: u64
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

struct ChunkFooter {
    data_size: usize,
    allocated: bool
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

pub struct Chunk {
    offset: u64,
    header: ChunkHeader,
}


impl Chunk {
    pub fn load(device: &Device, offset: u64) -> Option<Chunk> {
        match reader_at(fd, offset, CHUNK_HEADER_SIZE) {
            Ok(mut reader) => Some(Chunk {
                fd: fd,
                offset: offset,
                header: ChunkHeader::load(&mut reader)
            }),
            _ => None
        }
    }

    pub fn allocated(&self) -> bool {
        self.header.allocated
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn data_offset(&self) -> u64 {
        self.offset + CHUNK_HEADER_SIZE as u64
    }

    pub fn data_size(&self) -> usize {
        self.header.data_size
    }

    pub fn prev_free(&self, device: &Device) -> Option<Chunk> {
        match self.header.prev_free {
            0    => None,
            prev => Self::load(device, prev)
        }
    }

    pub fn next_free(&self, device: &Device) -> Option<Chunk> {
        match self.header.next_free {
            0    => None,
            next => Self::load(device, next)
        }
    }

    pub fn prev_free_in_order(&self) -> Option<Chunk> {
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

    pub fn next_free_in_order(&self) -> Option<Chunk> {
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