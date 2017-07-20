use std::io::{Read, Write, Cursor};
use std::vec::Vec;
use std::result;
use std::iter::{DoubleEndedIterator, Iterator};

use super::device;
use super::device::StorageDevice;
use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};

static CHUNK_HEADER_SIZE: usize = 3 * 8 + 1; // 3 * size_of::<u64>() + size_of::<u8>();
static CHUNK_FOOTER_SIZE: usize = 8 + 1;     // size_of::<u64>() + size_of::<u8>();

#[derive(Clone)]
struct ChunkHeader {
    data_size: usize,
    allocated: bool,
    prev_free: u64,
    next_free: u64
}

#[derive(Clone)]
struct ChunkFooter {
    data_size: usize,
    allocated: bool
}

#[derive(Clone)]
pub struct Chunk {
    offset: u64,
    header: ChunkHeader,
}

pub enum ChunkOperationError {
    DeviceError(device::DeviceError),
    OutOfRange,
    Allocated
}

pub type ChunkResult = result::Result<Chunk, ChunkOperationError>;
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
    fn _read_device(device: &StorageDevice, offset: u64, size: usize) -> device::Result<Buffer> {
        let mut buffer = vec![0u8; size];
        device.read_at(offset, buffer.as_mut_slice()).map(|_| Cursor::new(buffer))
    }

    pub fn load_from_device(device: &StorageDevice, offset: u64) -> device::Result<Chunk> {
        Self::_read_device(device, offset, CHUNK_HEADER_SIZE)
            .map(|mut reader| Chunk { offset: offset, header: ChunkHeader::load(&mut reader) })
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

    pub fn iter_freelist<'a>(&self, device: &'a StorageDevice) -> FreeListIterator<'a> {
        FreeListIterator {
            current: self.clone(),
            device: device
        }
    }

    pub fn iter_neighbours<'a>(&self, device: &'a StorageDevice) -> FreeNeighbourIterator<'a> {
        FreeNeighbourIterator {
            current: self.clone(),
            device: device
        }
    }
}

pub struct FreeListIterator<'a> {
    current: Chunk,
    device: &'a StorageDevice
}

pub struct FreeNeighbourIterator<'a> {
    current: Chunk,
    device: &'a StorageDevice
}

impl<'a> Iterator for FreeListIterator<'a> {
    type Item = device::Result<Chunk>;
    fn next(&mut self) -> Option<device::Result<Chunk>> {
        if self.current.header.next_free == 0 {
            None
        } else {
            let result = Chunk::load_from_device(
                self.device,
                self.current.header.next_free);

            match result {
                Ok(chunk) => {
                    self.current = chunk.clone();
                    Some(Ok(chunk))
                },
                _ => Some(result)
            }
        }
    }
}

impl<'a> DoubleEndedIterator for FreeListIterator<'a> {
    fn next_back(&mut self) -> Option<device::Result<Chunk>> {
        if self.current.header.prev_free == 0 {
            None
        } else {
            let result = Chunk::load_from_device(
                self.device,
                self.current.header.prev_free);

            match result {
                Ok(chunk) => {
                    self.current = chunk.clone();
                    Some(Ok(chunk))
                },
                _ => Some(result)
            }
        }
    }
}

impl<'a> Iterator for FreeNeighbourIterator<'a> {
    type Item = device::Result<Chunk>;
    fn next(&mut self) -> Option<device::Result<Chunk>> {
        let offset = self.current.offset +
            CHUNK_HEADER_SIZE as u64 +
            self.current.data_size() as u64 +
            CHUNK_FOOTER_SIZE as u64;

        match Chunk::load_from_device(self.device, offset) {
            Ok(Chunk { header: ChunkHeader { allocated: true, .. }, ..}) => None,
            Ok(chunk) => {
                self.current = chunk.clone();
                Some(Ok(chunk))
            },
            result => Some(result)
        }
    }
}

impl<'a> DoubleEndedIterator for FreeNeighbourIterator<'a> {
    fn next_back(&mut self) -> Option<device::Result<Chunk>> {
        let footer_offset = self.current.offset - CHUNK_FOOTER_SIZE as u64;
        let footer_result = Chunk::_read_device(self.device, footer_offset, CHUNK_FOOTER_SIZE)
            .map(|mut reader| ChunkFooter::load(&mut reader));

        match footer_result {
            Ok(footer) => {
                if !footer.allocated {
                    let location = footer_offset -
                        footer.data_size as u64 -
                        CHUNK_HEADER_SIZE as u64;

                    match Chunk::load_from_device(self.device, location) {
                        Ok(chunk) => {
                            self.current = chunk.clone();
                            Some(Ok(chunk))
                        },
                        result => Some(result)
                    }
                } else {
                    None
                }
            },
            Err(error) => Some(Err(error))
        }
    }
}