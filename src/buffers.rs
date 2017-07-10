use std::vec::Vec;

mod buffers {
    pub type RawBuffer = *mut *mut u8;

    struct BlockBuffer {
        _blocks: Vec<RawBuffer>,
        _buffer: Vec<u8>,
        _original_size: usize,
        _block_size: usize
    }

    impl BlockBuffer {
        pub fn new(size: usize, block_size: usize) -> BlockBuffer {
            let mut buffer = vec![0u8; size * block_size];
            BlockBuffer::from_buffer(buffer, block_size)
        }

        pub fn from_buffer(mut buffer: Vec<u8>, block_size: usize) -> BlockBuffer {
            let original_size = buffer.len();
            let block_count = original_size / block_size;
            let remaining = original_size - (size * block_count);

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
    }
}