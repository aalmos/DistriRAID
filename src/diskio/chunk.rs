use std::io::{Read, Write, Cursor};
use std::vec::Vec;
use super::device::{Result, StorageDevice};
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

static CHUNK_HEADER_SIZE: usize = 3 * 8 + 1; // 3 * size_of::<u64>() + size_of::<u8>();
static CHUNK_FOOTER_SIZE: usize = 8 + 1;     // size_of::<u64>() + size_of::<u8>();

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

pub struct Chunk {
    offset: u64,
    header: ChunkHeader,
}

pub type ChunkOnDevice = Result<Chunk>;
pub type ChunkOption = Option<Chunk>;
pub type Buffer = Cursor<Vec<u8>>;

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

impl Chunk {
    fn _read_device(device: &StorageDevice, offset: u64, size: usize) -> Result<Buffer> {
        let mut buffer = vec![0u8; size];
        device.read_at(offset, buffer.as_mut_slice()).map(|_| Cursor::new(buffer))
    }

    pub fn load_from_device(device: &StorageDevice, offset: u64) -> ChunkOnDevice {
        Self::_read_device(device, offset, CHUNK_HEADER_SIZE).map( |mut reader|
            Chunk {
                offset: offset,
                header: ChunkHeader::load(&mut reader)
            }
        )
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

    pub fn prev_free_on_device(&self, device: &StorageDevice) -> Option<ChunkOnDevice> {
        match self.header.prev_free {
            0    => None,
            prev => Some(Self::load_from_device(device, prev))
        }
    }

    pub fn next_free_on_device(&self, device: &StorageDevice) -> Option<ChunkOnDevice> {
        match self.header.next_free {
            0    => None,
            next => Some(Self::load_from_device(device, next))
        }
    }

    pub fn prev_free_in_order_from_device(&self, device: &StorageDevice) -> Result<ChunkOption> {
        let footer_offset = self.offset - CHUNK_FOOTER_SIZE as u64;
        Self::_read_device(device, footer_offset, CHUNK_FOOTER_SIZE)
            .map(|mut reader| ChunkFooter::load(&mut reader))
            .and_then(|footer| {
                if footer.allocated {
                    let location = footer_offset -
                        footer.data_size as u64 -
                        CHUNK_HEADER_SIZE as u64;
                    Chunk::load_from_device(device, location)
                        .map(|chunk| Some(chunk))
                } else {
                    Ok(None)
                }
            })
    }

    pub fn next_free_in_order_from_device(&self, device: &StorageDevice) -> Result<ChunkOption> {
        let offset = self.offset +
            CHUNK_HEADER_SIZE as u64 +
            self.data_size() as u64 +
            CHUNK_FOOTER_SIZE as u64;

        Self::load_from_device(device, offset).map(|chunk|
            if !chunk.allocated() {
                Some(chunk)
            } else {
                None
            }
        )
    }
}