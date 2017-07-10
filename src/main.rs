extern crate libc;

use libc::{c_int, calloc, malloc};
use std::io::Result;
use std::mem::size_of;
use std::vec::Vec;

type Schedule = *mut *mut c_int;
type BitMatrix = *mut c_int;
type BlockArray = *mut *mut u8;
type RawByteBuffer = *mut u8;

#[link(name = "Jerasure", kind = "static")]
#[link(name = "gf_complete", kind = "static")]
extern {
//    fn liberation_coding_bitmatrix(k: c_int, w: c_int) -> BitMatrix;

    fn liber8tion_coding_bitmatrix(k: c_int) -> BitMatrix;

    fn jerasure_print_bitmatrix(bitmatrix: BitMatrix, n: c_int, m: c_int, w: c_int);

//    fn jerasure_dumb_bitmatrix_to_schedule(k: c_int, m: c_int, w: c_int,
//                                           bitmatrix: BitMatrix) -> Schedule;

    fn jerasure_smart_bitmatrix_to_schedule(k: c_int, m: c_int, w: c_int,
                                            bitmatrix: BitMatrix) -> Schedule;

    fn jerasure_free_schedule(schedule: Schedule);

    fn jerasure_schedule_encode(k: c_int, m: c_int, w: c_int,
                                schedule: Schedule,
                                data_ptrs: BlockArray,
                                coding_ptrs: BlockArray,
                                size: c_int, packetsize: c_int);
}

struct Liber8tionCodec {
    _k: c_int,
    _w: c_int,
    _m: c_int,
    _packet_size: c_int,
    _bit_matrix: BitMatrix,
    _schedule: Schedule
}

impl Liber8tionCodec {

    fn _allocate_blocks(n: usize, size: usize) -> BlockArray
    {
        unsafe {
            let data = calloc(size_of::<*mut u8>(), n) as BlockArray;
            for i in 0..n {
                let block = data.offset(i as isize);
                std::ptr::write(block, calloc(size_of::<u8>(), size) as *mut u8);
            }
            return data;
        }
    }

    fn _free_blocks(data: BlockArray, n: usize)
    {
        unsafe {
            for i in 0..n {
                libc::free(data.offset(i as isize) as *mut libc::c_void)
            }
            libc::free(data as *mut libc::c_void)
        }
    }

    pub fn new() -> Liber8tionCodec {
        let m = 2;
        let w = 8;
        let k = 6;
        let packet_size = 8;

        unsafe {
            let bit_matrix = liber8tion_coding_bitmatrix(k as c_int);
            let schedule = jerasure_smart_bitmatrix_to_schedule(
                k as c_int, m as c_int, w as c_int, bit_matrix
            );
            // if (size%(k*w*packetsize*sizeof(long)) != 0) {
            Liber8tionCodec {
                _k: k as c_int,
                _w: w as c_int,
                _m: m as c_int,
                _packet_size: packet_size,
                _bit_matrix: bit_matrix,
                _schedule: schedule
            }
        }
    }

    pub fn encode(&self, input: &[u8], output: &mut [u8])// -> Result<usize>
    {
        let word_size = size_of::<*mut u8>();
        let size = input.len() * word_size / self._k as usize;

        let chunk_size = (self._k * self._w * self._packet_size * word_size as i32) as usize;
        let mut new_size = size;

        while new_size % chunk_size != 0 {
            new_size += 1;
        }

        let block_size = new_size / self._k as usize;

        unsafe {
            let mut fake_data = calloc(size_of::<u8>(), new_size) as *mut u8;
            let mut data = calloc(size_of::<*mut u8>(), self._k as usize) as BlockArray;
            let mut coding = Liber8tionCodec::_allocate_blocks(self._m as usize, block_size);
            let mut input_ptr = fake_data;//input.as_ptr();

            for i in 0..self._k {
                std::ptr::write(data.offset(i as isize), input_ptr as *mut u8);
                input_ptr = input_ptr.offset(block_size as isize);
            }

            jerasure_schedule_encode(
                self._k, self._m, self._w, self._schedule,
                data, coding, block_size as c_int,
                self._packet_size
            );

            Liber8tionCodec::_free_blocks(coding, self._m as usize);
            libc::free(data as *mut libc::c_void);
            libc::free(fake_data as *mut libc::c_void);
        }
        // if (size%(k*w*packetsize*sizeof(long)) != 0) {

        //jerasure_schedule_encode(self._k, self._m, self._w, )
    }

    pub fn print(&self) {
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

fn main() {
    let codec = Liber8tionCodec::new();
    codec.print();
    let mut output = [1u8, 10];
    let input = [0u8, 10];

    codec.encode(&input, &mut output);

    //unsafe {

//        let mut bitmatrix = liberation_coding_bitmatrix(k, w);
//        jerasure_print_bitmatrix(bitmatrix, w*m, w*k, w);
//        libc::free(bitmatrix as *mut libc::c_void);
//    }
}
