use std::vec::Vec;

pub type RawBuffer = *mut u8;

pub struct BlockBuffer {
    _blocks: Vec<RawBuffer>,
    _buffer: Vec<u8>,
    _original_size: usize,
    _block_size: usize
}

impl BlockBuffer {
    pub fn new(block_count: usize, block_size: usize) -> BlockBuffer {
        let mut buffer = vec![0u8; block_count * block_size];
        BlockBuffer::from_raw_padded(buffer, block_count, block_size)
    }

    pub fn from_raw_fitting(mut buffer: Vec<u8>, block_size: usize) -> BlockBuffer {
        let mut block_count = buffer.len() / block_size;

        if block_count * block_size < buffer.len() {
            block_count += 1;
        }

        BlockBuffer::from_raw_padded(buffer, block_count, block_size)
    }

    pub fn from_raw_padded(mut buffer: Vec<u8>, block_count: usize, block_size: usize) -> BlockBuffer {
        assert!(buffer.len() <= block_count * block_size, "input buffer exceeds given size");

        let desired_size = block_count * block_size;
        let original_size = buffer.len();
        let remaining = desired_size - original_size;

        if remaining > 0 {
            buffer.append(&mut vec![0u8; remaining]);
        }

        buffer.shrink_to_fit();

        BlockBuffer {
            _blocks: buffer.chunks_mut(block_size).map(|x| x.as_mut_ptr()).collect(),
            _buffer: buffer,
            _original_size: original_size,
            _block_size: block_size
        }
    }

    pub fn as_ptr(&self) -> *const *mut u8 {
        self._blocks.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut *mut u8 {
        self._blocks.as_mut_ptr()
    }

    pub fn block_size(&self) -> usize {
        self._block_size
    }

    pub fn block_count(&self) -> usize {
        self._blocks.len()
    }
}