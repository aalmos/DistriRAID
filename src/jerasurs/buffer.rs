use std::vec::Vec;
use std::ops::Index;
use std::option::Option;
use std::slice;
use std::mem;

use libc::{free, c_void};
use super::native::RawBlockBuffer;

pub struct Block {
    _id: usize,
    _data: *mut u8,
    _size: usize,
    _owned: bool,
}

impl Block {
    pub fn new(id: usize, data: &[u8]) -> Block {
        let mut data_vec = data.to_vec();
        data_vec.shrink_to_fit();

        let result = Block {
            _id: id,
            _data: data_vec.as_mut_ptr(),
            _size: data_vec.len(),
            _owned: true
        };

        mem::forget(data_vec);

        result
    }

    fn for_buffer_view(id: usize, data: *mut u8, size: usize) -> Block {
        Block {
            _id: id,
            _data: data,
            _size: size,
            _owned: false
        }
    }

    pub fn id(&self) -> usize {
        self._id
    }

    pub fn data(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self._data, self._size)
        }
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        if self._owned {
            unsafe {
                free(self._data as *mut c_void);
            }
        }
    }
}

pub struct BlockBuffer {
    _buffer: Vec<u8>,
    _data_size: usize,
    _is_data_accessible: bool,

    _blocks: Vec<Option<Block>>,
    _block_size: usize,

    _data_block_count: usize,
    _parity_block_count: usize
}

impl BlockBuffer {
    pub fn from_data_buffer(mut data_buffer: Vec<u8>,
                            block_size: usize,
                            data_block_count: usize,
                            parity_block_count: usize) -> BlockBuffer {
        let original_data_size = data_buffer.len();
        let total_block_count = data_block_count + parity_block_count;
        let desired_buffer_size = total_block_count * block_size;
        let padding_size = desired_buffer_size - original_data_size;

        data_buffer.append(&mut vec![0u8; padding_size]);
        data_buffer.shrink_to_fit();

        let all_blocks = data_buffer.chunks_mut(block_size)
            .zip(0..total_block_count)
            .map(|(c, i)| Some(Block::for_buffer_view(i, c.as_mut_ptr(), block_size)))
            .collect::<Vec<_>>();

        BlockBuffer {
            _buffer: data_buffer,
            _data_size: original_data_size,
            _is_data_accessible: true,
            _blocks: all_blocks,
            _block_size: block_size,
            _data_block_count: data_block_count,
            _parity_block_count: parity_block_count
        }
    }

    pub fn data_ptrs(&mut self) -> Vec<*mut u8> {
        self._buffer.chunks_mut(self._block_size)
            .map(|c| c.as_mut_ptr())
            .take(self._data_block_count)
            .collect()
    }

    pub fn parity_ptrs(&mut self) -> Vec<*mut u8> {
        self._buffer.chunks_mut(self._block_size)
            .map(|c| c.as_mut_ptr())
            .skip(self._parity_block_count)
            .collect()
    }

    pub fn block_size(&self) -> usize {
        self._block_size
    }

    pub fn data_size(&self) -> usize {
        self._data_size
    }

    pub fn blocks(&self) -> &Vec<Option<Block>> {
        &self._blocks
    }

    pub fn data(&self) -> Option<&[u8]> {
        match self._is_data_accessible {
            true => Some(&self._buffer[0..self._data_size]),
            _    => None
        }
    }
}

//impl<'a> Index<usize> for BlockBuffer<'a> {
//    type Output = Option<&'a[u8]>;
//
//    fn index(&self, index: usize) -> &Self::Output {
//        assert!(index < self._all_blocks_view.len());
//        &self._all_blocks_view[index]
//    }
//}
