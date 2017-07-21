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

#[derive(Clone)]
pub struct FreeList<'a> {
    head: Chunk,
    device: &'a StorageDevice
}

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

    pub fn remove_from_freelist(&mut self, device: &mut StorageDevice) -> device::Result<()> {
        let mut prev = Self::load_from_device(device, self.header.prev_free)?;
        let mut next = Self::load_from_device(device, self.header.next_free)?;

        next.header.prev_free = prev.offset;
        prev.header.next_free = next.offset;

        self.header.allocated = true;

        self.dump_to_device(device)?;
        prev.dump_to_device(device)?;
        next.dump_to_device(device)?;

        Ok(())
    }

    pub fn coalesce_two(mut first: Chunk, mut second: Chunk, device: &mut StorageDevice) -> device::Result<Chunk> {
        if !first.allocated() {
            let mut prev = Self::load_from_device(device, second.header.prev_free)?;
            let mut next = Self::load_from_device(device, second.header.next_free)?;

            prev.header.next = first.offset;
            next.header.prev = first.offset;

            first.header.data_size += second.data_size() + CHUNK_HEADER_SIZE;
            first.dump_to_device(device)?;

            Ok(first)
        } else {
            second.remove_from_freelist(device)?;

        }
    }

    pub fn coalesce_three(first: Chunk, second: Chunk, third: Chunk) {

    }

    pub fn load_from_device(device: &StorageDevice, offset: u64) -> device::Result<Chunk> {
        Self::_read_device(device, offset, CHUNK_HEADER_SIZE)
            .map(|mut reader| Chunk { offset: offset, header: ChunkHeader::load(&mut reader) })
    }

    pub fn dump_to_device(&self, device: &mut StorageDevice) -> device::Result<()> {
        let mut writer = Cursor::new(vec![0u8; CHUNK_HEADER_SIZE]);
        ChunkHeader::dump(&self.header, &mut writer);
        device.write_at(self.offset, writer.get_ref().as_slice())?;

        let footer_offset = self.offset + CHUNK_HEADER_SIZE as u64 + self.data_size() as u64;
        writer = Cursor::new(vec![0u8; CHUNK_FOOTER_SIZE]);

        ChunkFooter::dump(&ChunkFooter {
                allocated: self.allocated(),
                data_size: self.data_size()
            }, &mut writer);

        device.write_at(footer_offset, writer.get_ref().as_slice())?;

        Ok(())
    }

    pub fn as_freelist<'a>(&self, device: &'a StorageDevice) -> FreeList<'a> {
        FreeList::new(self.clone(), device)
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

    pub fn unallocated_neighbours(&self, device: &StorageDevice)
        -> device::Result<(Option<Chunk>, Option<Chunk>)> {

        let prev = self._peek_back(device)?;
        let next = self._peek_forward(device)?;

        match (prev.allocated(), next.allocated()) {
            (false, false) => Ok((Some(prev), Some(next))),
            (false, true) => Ok((Some(prev), None)),
            (true, false) => Ok((None, Some(next))),
            (true, true) => Ok((None, None))
        }
    }

    fn _peek_forward(&self, device: &StorageDevice) -> device::Result<Chunk> {
        let offset = self.offset +
            CHUNK_HEADER_SIZE as u64 +
            self.data_size() as u64 +
            CHUNK_FOOTER_SIZE as u64;
        Chunk::load_from_device(device, offset)
    }

    fn _peek_back(&self, device: &StorageDevice) -> device::Result<Chunk> {
        let footer_offset = self.offset - CHUNK_FOOTER_SIZE as u64;
        Chunk::_read_device(device, footer_offset, CHUNK_FOOTER_SIZE)
            .map(|mut reader| ChunkFooter::load(&mut reader))
            .and_then(|footer| {
                if footer.allocated {
                    return Ok(Chunk {
                        offset: 0,
                        header: ChunkHeader {
                            prev_free: 0,
                            next_free: 0,
                            allocated: true,
                            data_size: footer.data_size
                        }
                    })
                }

                let location = footer_offset -
                    footer.data_size as u64 -
                    CHUNK_HEADER_SIZE as u64;

                Chunk::load_from_device(device, self.offset)
            })
    }
}

impl<'a> FreeList<'a> {
    pub fn new(chunk: Chunk, device: &StorageDevice) -> FreeList {
        FreeList {
            head: chunk,
            device: device
        }
    }

    pub fn head(&self) -> &Chunk {
        &self.head
    }

    pub fn next(&self) -> Option<device::Result<FreeList<'a>>> {
        if self.head.header.next_free == 0 {
            None
        } else {
            Some(Chunk::load_from_device(
                self.device,
                self.head.header.next_free
            ).map(|chunk| Self::new(chunk, self.device)))
        }
    }

    pub fn next_back(&self) -> Option<device::Result<FreeList<'a>>> {
        if self.head.header.prev_free == 0 {
            None
        } else {
            Some(Chunk::load_from_device(
                self.device,
                self.head.header.prev_free
            ).map(|chunk| Self::new(chunk, self.device)))
        }
    }
}

pub struct FreeListIterator<'a> {
    current: FreeList<'a>,
    failed: bool,
    first: bool,
}

impl<'a> FreeListIterator<'a> {
    fn _process_item(&mut self, item: Option<device::Result<FreeList<'a>>>)
        -> Option<device::Result<Chunk>> {

        match item {
            Some(Ok(next)) => {
                self.current = next;
                Some(Ok(self.current.head.clone()))
            },
            Some(Err(error)) => {
                self.failed = true;
                Some(Err(error))
            },
            _ => None
        }
    }
}

pub struct FreeNeighbourIterator<'a> {
    current: Chunk,
    device: &'a StorageDevice,
    failed: bool
}

impl<'a> Iterator for FreeListIterator<'a> {
    type Item = device::Result<Chunk>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            None
        } else if self.first {
            self.first = false;
            Some(Ok(self.current.head.clone()))
        } else {
            let next = self.current.next();
            self._process_item(next)
        }
    }
}

impl<'a> DoubleEndedIterator for FreeListIterator<'a> {
    fn next_back(&mut self) -> Option<device::Result<Chunk>> {
        if self.failed {
            None
        } else if self.first {
            self.first = false;
            Some(Ok(self.current.head.clone()))
        } else {
            let prev = self.current.next_back();
            self._process_item(prev)
        }
    }
}
//
//impl<'a> Iterator for FreeNeighbourIterator<'a> {
//    type Item = device::Result<Chunk>;
//    fn next(&mut self) -> Option<device::Result<Chunk>> {
//        if self.failed {
//            return None;
//        }
//
//        let offset = self.current.offset +
//            CHUNK_HEADER_SIZE as u64 +
//            self.current.data_size() as u64 +
//            CHUNK_FOOTER_SIZE as u64;
//
//        match Chunk::load_from_device(self.device, offset) {
//            Ok(Chunk { header: ChunkHeader { allocated: true, .. }, ..}) => None,
//            Ok(chunk) => {
//                self.current = chunk.clone();
//                Some(Ok(chunk))
//            },
//            error @ Err(_) => {
//                self.failed = true;
//                Some(error)
//            }
//        }
//    }
//}
//
//impl<'a> DoubleEndedIterator for FreeNeighbourIterator<'a> {
//    fn next_back(&mut self) -> Option<device::Result<Chunk>> {
//        if self.failed {
//            return None;
//        }
//
//        let footer_offset = self.current.offset - CHUNK_FOOTER_SIZE as u64;
//        let footer_result = Chunk::_read_device(self.device, footer_offset, CHUNK_FOOTER_SIZE)
//            .map(|mut reader| ChunkFooter::load(&mut reader));
//
//        match footer_result {
//            Ok(footer) => {
//                if footer.allocated {
//                    return None
//                }
//
//                let location = footer_offset -
//                    footer.data_size as u64 -
//                    CHUNK_HEADER_SIZE as u64;
//
//                match Chunk::load_from_device(self.device, location) {
//                    Ok(chunk) => {
//                        self.current = chunk.clone();
//                        Some(Ok(chunk))
//                    },
//                    error @ Err(_) => {
//                        self.failed = true;
//                        Some(error)
//                    }
//                }
//            },
//            Err(error) => {
//                self.failed = true;
//                Some(Err(error))
//            }
//        }
//    }
//}