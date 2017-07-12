use std::vec::Vec;
use std::ops::Index;
use std::slice;

pub struct BlockBuffer {
    _data_blocks: Vec<*mut u8>,
    _parity_blocks: Vec<*mut u8>,
    _buffer: Vec<u8>,
    _data_size: usize,
    _block_size: usize,
}

impl BlockBuffer {
    pub fn from_data_buffer(mut data_buffer: Vec<u8>,
                            block_size: usize,
                            data_block_count: usize,
                            parity_block_count: usize) -> BlockBuffer {
        let original_data_size = data_buffer.len();
        let desired_buffer_size = (data_block_count + parity_block_count) * block_size;
        let padding_size = desired_buffer_size - original_data_size;

        data_buffer.append(&mut vec![0u8; padding_size]);
        data_buffer.shrink_to_fit();

        let all_blocks = data_buffer
            .chunks_mut(block_size)
            .map(|x| x.as_mut_ptr())
            .collect::<Vec<_>>();

        let (data_blocks, parity_blocks) = all_blocks.split_at(data_block_count);

        BlockBuffer {
            _data_blocks: data_blocks.to_vec(),
            _parity_blocks: parity_blocks.to_vec(),
            _buffer: data_buffer,
            _data_size: original_data_size,
            _block_size: block_size
        }
    }

    pub fn data_blocks(&self) -> &[*mut u8] {
        &self._data_blocks
    }

    pub fn data_blocks_mut(&mut self) -> &mut [*mut u8] {
        &mut self._data_blocks
    }

    pub fn parity_blocks(&self) -> &[*mut u8] {
        &self._parity_blocks
    }

    pub fn parity_blocks_mut(&mut self) -> &mut [*mut u8] {
        &mut self._parity_blocks
    }

    pub fn block_size(&self) -> usize {
        self._block_size
    }

    pub fn total_block_count(&self) -> usize {
        self._data_blocks.len() + self._parity_blocks.len()
    }
}

impl Index<usize> for BlockBuffer {
    type Output = [u8];

    fn index(&self, index: usize) -> &[u8] {
        assert!(index < self._data_blocks.len() + self._parity_blocks.len());

        unsafe {
            if index < self._data_blocks.len() {
                slice::from_raw_parts(
                    self._data_blocks[index],
                    self._block_size
                )
            } else {
                slice::from_raw_parts(
                    self._parity_blocks[index - self._data_blocks.len()],
                    self._block_size
                )
            }
        }
    }
}
