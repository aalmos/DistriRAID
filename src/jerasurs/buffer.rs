use std::vec::Vec;
use std::ops::Index;
use std::slice;

pub struct BlockBuffer<'a> {
    _data_blocks: Vec<*mut u8>,
    _parity_blocks: Vec<*mut u8>,
    _all_blocks_view: Vec<&'a[u8]>,
    _buffer: Vec<u8>,
    _data_size: usize,
    _block_size: usize,
}

impl<'a> BlockBuffer<'a> {
    pub fn from_data_buffer(mut data_buffer: Vec<u8>,
                            block_size: usize,
                            data_block_count: usize,
                            parity_block_count: usize) -> BlockBuffer<'a> {
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
        let all_blocks_view: Vec<_> = data_blocks.iter()
            .chain(parity_blocks.iter())
            .map(|p| unsafe { slice::from_raw_parts(*p, block_size) })
            .collect::<Vec<&[u8]>>();

        BlockBuffer {
            _data_blocks: data_blocks.to_vec(),
            _parity_blocks: parity_blocks.to_vec(),
            _all_blocks_view: all_blocks_view,
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

impl<'a> Index<usize> for BlockBuffer<'a> {
    type Output = &'a[u8];

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self._all_blocks_view.len());
        &self._all_blocks_view[index]
    }
}
