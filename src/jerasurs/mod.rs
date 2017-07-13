mod native;
pub mod buffer;
pub mod codecs;

use libc::{c_int, free, c_void};

use self::native::BitMatrix;
use self::native::Schedule;
use self::native::ScheduleCache;

use self::buffer::BlockBuffer;

static WORD_SIZE: i32 = 8;

pub struct Codec {
    _k: c_int,
    _w: c_int,
    _m: c_int,
    _packet_size: c_int,
    _bit_matrix: BitMatrix,
    _schedule: Schedule,
    _schedule_cache: ScheduleCache,
    _encoding_technique: fn(&Codec, &mut BlockBuffer),
    _decoding_technique: fn(&Codec, &mut BlockBuffer) -> bool
}

impl Codec {
    pub fn encode(&self, input: &[u8]) -> BlockBuffer {
        let input_vec = input.to_vec();
        let padding_size = input_vec.len() % self.chunk_size();
        let block_size = (input_vec.len() + padding_size) as usize / self.data_block_count();

        let mut result = BlockBuffer::from_data_buffer(
            input_vec, block_size,
            self.data_block_count(),
            self.parity_block_count()
        );

        (self._encoding_technique)(self, &mut result);

        result
    }

    pub fn decode<'a>(&self, input: &'a mut BlockBuffer) -> Option<&'a [u8]> {
        if !(self._decoding_technique)(self, input) {
            return None;
        }
        
        input.data()
    }

    pub fn data_block_count(&self) -> usize {
        self._k as usize
    }

    pub fn parity_block_count(&self) -> usize {
        self._m as usize
    }

    pub fn total_block_count(&self) -> usize {
        (self._m + self._k) as usize
    }

    pub fn chunk_size(&self) -> usize {
        (self._k * self._w * self._packet_size * WORD_SIZE) as usize
    }

    pub fn print_bit_matrix(&self) {
        unsafe {
            native::jerasure_print_bitmatrix(
                self._bit_matrix,
                self._w * self._m,
                self._w * self._k,
                self._w
            );
        }
    }
}

impl Drop for Codec {
    fn drop(&mut self) {
        unsafe {
            free(self._bit_matrix as *mut c_void);
            native::jerasure_free_schedule(self._schedule);
            native::jerasure_free_schedule_cache(self._k, self._m, self._schedule_cache);
        }
    }
}
