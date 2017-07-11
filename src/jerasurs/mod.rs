extern crate libc;

mod native;

use libc::c_int;
use std::vec::Vec;
use native::BitMatrix;
use native::Schedule;

pub struct Codec {
    _k: c_int,
    _w: c_int,
    _m: c_int,
    _packet_size: c_int,
    _bit_matrix: BitMatrix,
    _schedule: Schedule
}

impl Codec {
    fn encode(&self, input: &[u8]) -> BlockBuffer {
        let mut input_vec = input.to_vec();
        let padding_size = input_vec.len() % self.chunk_size();
        let block_size = (input_vec.len() + padding_size) as usize / self.data_block_count();

        let mut result = BlockBuffer::from_data_buffer(
            input_vec, block_size,
            self.data_block_count(),
            self.parity_block_count()
        );

        unsafe {
            native::jerasure_schedule_encode(
                self._k, self._m, self._w, self._schedule,
                result.data_blocks_mut().as_mut_ptr(),
                result.parity_blocks_mut().as_mut_ptr(),
                block_size as c_int,
                self._packet_size
            );
        }

        result
    }

    fn decode(&self, input: &BlockBuffer) -> Vec<u8> {
        unimplemented!()
    }

    fn data_block_count(&self) -> usize {
        self._k as usize
    }

    fn parity_block_count(&self) -> usize {
        self._m as usize
    }

    fn total_block_count(&self) -> usize {
        (self._m + self._k) as usize
    }

    fn chunk_size(&self) -> usize {
        (self._k * self._w * self._packet_size * WORD_SIZE) as usize
    }
}

impl Drop for Codec {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._bit_matrix as *mut libc::c_void);
            native::jerasure_free_schedule(self._schedule);
        }
    }
}