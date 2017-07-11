extern crate libc;

use buffers::BlockBuffer;
use std::mem::size_of;
use libc::c_int;
use std::vec::Vec;

static LIBER8TION_W: i32 = 8;
static LIBER8TION_K: i32 = 7;
static LIBER8TION_M: i32 = 2;
static LIBER8TION_PACKET_SIZE: i32 = 64;

static WORD_SIZE: i32 = 8;

type Schedule = *mut *mut c_int;
type BitMatrix = *mut c_int;
type RawBlockBuffer = *mut *mut u8;

#[link(name = "Jerasure")]
#[link(name = "gf_complete")]
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

pub fn encode_simple(data: &[u8]) {
    let w = 8;
    let m = 2;
    
    let k = 6;

    let packet_size = 64;

    let mut new_size = data.len();

    let min_data_size = k * w * packet_size * 8;

    println!("min_data_size {:?}", min_data_size);

    if data.len() % (k * w * packet_size * 8) != 0 {
        while new_size % (k * w * packet_size * 8) != 0 {
            new_size += 1;
        }
    }

    let block_size = new_size / k;
    let padding_size = new_size - data.len();

    let mut data_vec = data.to_vec();
    let mut parity_vec = vec![0u8; m * block_size];

    data_vec.append(&mut vec![0u8; padding_size]);
    
    unsafe {
        let mut bit_matrix = liber8tion_coding_bitmatrix(k as i32);
        let mut schedule = jerasure_smart_bitmatrix_to_schedule(k as i32, m as i32, w as i32, bit_matrix);
    
        let mut data_ptrs: Vec<_> = data_vec.chunks_mut(block_size).map(|x| x.as_mut_ptr()).collect();
        let mut parity_ptrs: Vec<_> = parity_vec.chunks_mut(block_size).map(|x| x.as_mut_ptr()).collect();

        jerasure_schedule_encode(k as i32, m as i32, w as i32, schedule, data_ptrs.as_mut_ptr(), parity_ptrs.as_mut_ptr(), block_size as i32, packet_size as i32);
        
        println!("DATA");
        
        for chunk in data_vec.chunks(block_size) {
            println!("{:?}", chunk);
        }

        println!("PARITY");
        
        for chunk in parity_vec.chunks(block_size) {
            println!("{:?}", chunk);
        }
    }
}

pub trait Codec {
    fn data_block_count(&self) -> usize;
    fn parity_block_count(&self) -> usize;
    fn total_block_count(&self) -> usize;
    fn chunk_size(&self) -> usize;

    fn encode(&self, input: &[u8]) -> BlockBuffer;
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
            jerasure_schedule_encode(
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
