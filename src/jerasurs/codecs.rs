pub mod liber8tion {
    use super::super::Codec;
    use super::super::native;
    use super::super::buffer::BlockBuffer;
    use libc::c_int;

    static LIBER8TION_W: i32 = 8;
    static LIBER8TION_M: i32 = 2;

    pub fn create(k: u32, packet_size: usize) -> Codec {
        assert!(packet_size > 0);
        assert!(k <= LIBER8TION_W as u32);

        unsafe {
            let bit_matrix = native::liber8tion_coding_bitmatrix(k as c_int);
            let schedule = native::jerasure_smart_bitmatrix_to_schedule(
                k as c_int, LIBER8TION_M, LIBER8TION_W, bit_matrix
            );

            let schedule_cache = native::jerasure_generate_schedule_cache(
                k as c_int, LIBER8TION_M, LIBER8TION_W,
                bit_matrix, 1
            );

            Codec {
                _k: k as c_int,
                _w: LIBER8TION_W,
                _m: LIBER8TION_M,
                _packet_size: packet_size as c_int,
                _bit_matrix: bit_matrix,
                _schedule: schedule,
                _schedule_cache: schedule_cache,
                _encoding_technique: encode,
                _decoding_technique: decode
            }
        }
    }

    fn encode(codec: &Codec, buffer: &mut BlockBuffer) {
        unsafe {
            native::jerasure_schedule_encode(
                codec._k, codec._m, codec._w, codec._schedule,
                buffer.data_ptrs(),
                buffer.parity_ptrs(),
                buffer.block_size() as c_int,
                codec._packet_size
            );
        }
    }

    fn decode(codec: &Codec, buffer: &mut BlockBuffer) -> bool {
        let mut erasures = Vec::<c_int>::new();

        for (block, id) in buffer.blocks().iter().zip(0..buffer.blocks().len()) {
            if block.is_none() {
                erasures.push(id as c_int);
            }
        }

        erasures.push(-1);

        unsafe {
            let result = native::jerasure_schedule_decode_cache(
                codec._k, codec._m, codec._w,
                codec._schedule_cache,
                erasures.as_mut_ptr(),
                buffer.data_ptrs(),
                buffer.parity_ptrs(),
                buffer.block_size() as c_int,
                codec._packet_size
            );

            if result != 0 {
                return false;
            }

            for id in erasures[0..erasures.len() - 1].iter() {
                buffer.mark_block_as_restored(*id as usize);
            }

            true
        }
    }
}