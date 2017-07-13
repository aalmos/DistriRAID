use std::vec::Vec;
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

    pub fn clone_from_slice(&mut self, data: &[u8]) {
        assert!(data.len() <= self._size);
        self.data_mut().clone_from_slice(data);
    }

    pub fn data(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self._data, self._size)
        }
    }

    pub fn data_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self._data, self._size)
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
    _blocks_raw: Vec<*mut u8>,
    _block_size: usize,

    _data_block_count: usize,
    _parity_block_count: usize
}

impl BlockBuffer {
    pub fn from_data_buffer(mut data_buffer: Vec<u8>,
                            block_size: usize,
                            data_block_count: usize,
                            parity_block_count: usize) -> BlockBuffer {

        assert!(data_buffer.len() <= block_size * data_block_count);

        let original_data_size = data_buffer.len();
        let total_block_count = data_block_count + parity_block_count;
        let desired_buffer_size = total_block_count * block_size;
        let padding_size = desired_buffer_size - original_data_size;

        data_buffer.append(&mut vec![0u8; padding_size]);
        data_buffer.shrink_to_fit();

        let mut blocks_raw = data_buffer.chunks_mut(block_size)
            .map(|c| c.as_mut_ptr())
            .collect::<Vec<_>>();

        let blocks = blocks_raw.iter_mut().zip(0..)
            .map(|(p, i)| Some(Block::for_buffer_view(i, *p, block_size)))
            .collect::<Vec<_>>();

        BlockBuffer {
            _buffer: data_buffer,
            _data_size: original_data_size,
            _is_data_accessible: true,
            _blocks: blocks,
            _blocks_raw: blocks_raw,
            _block_size: block_size,
            _data_block_count: data_block_count,
            _parity_block_count: parity_block_count
        }
    }

    pub fn from_blocks(blocks: &[Block],
                       block_size: usize,
                       data_block_count: usize,
                       parity_block_count: usize,
                       data_size: usize) -> BlockBuffer {

        let block_count = data_block_count + parity_block_count;

        assert!(blocks.iter().all(|b| b.data().len() == block_size));
        assert!(blocks.len() <= block_count);

        let mut buffer = vec![0u8; block_count * block_size];
        buffer.shrink_to_fit();

        let blocks_raw = buffer.chunks_mut(block_size)
            .map(|c| c.as_mut_ptr())
            .collect::<Vec<_>>();

        let mut block_opts = blocks_raw.iter().map(|_| None).collect::<Vec<Option<Block>>>();

        let mut data_blocks_found = 0;

        for block in blocks.iter() {
            unsafe {
                assert!(block_opts[block.id()].is_none());

                if block.id() < data_block_count {
                    data_blocks_found += 1;
                }

                let mut new_block = Block::for_buffer_view(
                    block.id(),
                    buffer.as_mut_ptr().offset((block.id() * block_size) as isize),
                    block_size
                );

                new_block.clone_from_slice(block.data());

                block_opts[block.id()] = Some(new_block);
            }
        }

        BlockBuffer {
            _buffer: buffer,
            _data_size: data_size,
            _is_data_accessible: data_blocks_found == data_block_count,
            _blocks: block_opts,
            _blocks_raw: blocks_raw,
            _block_size: block_size,
            _data_block_count: data_block_count,
            _parity_block_count: parity_block_count
        }
    }

    pub unsafe fn data_ptrs(&mut self) -> RawBlockBuffer {
        self._blocks_raw.as_mut_ptr()
    }

    pub unsafe fn parity_ptrs(&mut self) -> RawBlockBuffer {
        self._blocks_raw.as_mut_ptr().offset(self._data_block_count as isize)
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

    pub fn mark_present(&mut self, id: usize) {
        self._blocks[id] = Some(Block::for_buffer_view(id, self._blocks_raw[id], self._block_size));
    }

    pub fn mark_absent(&mut self, id: usize) {
        self._blocks[id] = None;
    }

    pub fn overwrite_block(&mut self, id: usize, data: &[u8]) {
        assert!(data.len() <= self._block_size);

        let start = id * self._block_size;
        let end = start + data.len();
        self._buffer[start..end].clone_from_slice(data);
    }
}