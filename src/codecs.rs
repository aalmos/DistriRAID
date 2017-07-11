extern crate libc;

use buffers::BlockBuffer;
use std::mem::size_of;
use libc::c_int;
use std::vec::Vec;

static LIBER8TION_W: i32 = 8;
static LIBER8TION_K: i32 = 6;
static LIBER8TION_M: i32 = 2;
static LIBER8TION_PACKET_SIZE: i32 = 8;

static WORD_SIZE: i32 = 8;

type Schedule = *mut *mut c_int;
type BitMatrix = *mut c_int;
type RawBlockBuffer = *mut *mut u8;

#[link(name = "Jerasure", kind = "static")]
#[link(name = "gf_complete", kind = "static")]
extern {
    fn liber8tion_coding_bitmatrix(k: c_int) -> BitMatrix;

    fn jerasure_print_bitmatrix(
        bit_matrix: BitMatrix,
        n: c_int, m: c_int, w: c_int
    );

    fn jerasure_smart_bitmatrix_to_schedule(
        k: c_int, m: c_int, w: c_int,
        bit_matrix: BitMatrix
    ) -> Schedule;

    fn jerasure_free_schedule(schedule: Schedule);

    fn jerasure_schedule_encode(
        k: c_int, m: c_int, w: c_int,
        schedule: Schedule,
        data_in: RawBlockBuffer,
        coding_out: RawBlockBuffer,
        block_size: c_int,
        packet_size: c_int
    );
}

pub trait Codec {
    fn data_block_count(&self) -> usize;
    fn parity_block_count(&self) -> usize;
    fn total_block_count(&self) -> usize;
    fn chunk_size(&self) -> usize;

    fn encode(&self, input: &[u8]) -> (BlockBuffer, BlockBuffer);
    fn decode(&self, input: &BlockBuffer) -> Vec<u8>;
}

pub struct Liber8tionCodec {
    _k: c_int,
    _w: c_int,
    _m: c_int,
    _packet_size: c_int,
    _bit_matrix: BitMatrix,
    _schedule: Schedule
}

impl Liber8tionCodec {
    pub fn new() -> Liber8tionCodec {
        unsafe {
            let bit_matrix = liber8tion_coding_bitmatrix(LIBER8TION_K);
            let schedule = jerasure_smart_bitmatrix_to_schedule(
                LIBER8TION_K, LIBER8TION_M, LIBER8TION_W, bit_matrix
            );

            Liber8tionCodec {
                _k: LIBER8TION_K,
                _w: LIBER8TION_W,
                _m: LIBER8TION_M,
                _packet_size: LIBER8TION_PACKET_SIZE,
                _bit_matrix: bit_matrix,
                _schedule: schedule
            }
        }
    }

    pub fn print_bit_matrix(&self) {
        unsafe {
            jerasure_print_bitmatrix(
                self._bit_matrix,
                self._w * self._m,
                self._w * self._k,
                self._w
            );
        }
    }
}

impl Drop for Liber8tionCodec {
    fn drop(&mut self) {
        unsafe {
            libc::free(self._bit_matrix as *mut libc::c_void);
            jerasure_free_schedule(self._schedule);
        }
    }
}

impl Codec for Liber8tionCodec {
    fn encode(&self, input: &[u8]) -> (BlockBuffer, BlockBuffer) {
        let mut input_vec = input.to_vec();
        let chunk_size = self.chunk_size();
        let padding_size = input_vec.len() % self.chunk_size();
        let block_size = (input_vec.len() + padding_size) as usize / self.data_block_count();

        let mut data_blocks = BlockBuffer::from_raw_padded(
            input_vec,
            self.data_block_count(),
            block_size
        );

        let mut parity_blocks = BlockBuffer::new(self.parity_block_count(), block_size);

        unsafe {
            jerasure_schedule_encode(
                self._k, self._m, self._w, self._schedule,
                data_blocks.as_mut_ptr(),
                parity_blocks.as_mut_ptr(), block_size as c_int,
                self._packet_size
            );
        }

        (data_blocks, parity_blocks)
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
        (self._k * self._w * self._packet_size) as usize
    }
}
