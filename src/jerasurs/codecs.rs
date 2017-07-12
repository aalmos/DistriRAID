pub mod liber8tion {
    use super::super::Codec;
    use super::super::native;
    use libc::c_int;

    static LIBER8TION_W: i32 = 8;
    static LIBER8TION_M: i32 = 2;

    pub fn create(k: u32, packet_size: usize) -> Codec {
        unsafe {
            let bit_matrix = native::liber8tion_coding_bitmatrix(k as c_int);
            let schedule = native::jerasure_smart_bitmatrix_to_schedule(
                k as c_int, LIBER8TION_M, LIBER8TION_W, bit_matrix
            );

            Codec {
                _k: k as c_int,
                _w: LIBER8TION_W,
                _m: LIBER8TION_M,
                _packet_size: packet_size as c_int,
                _bit_matrix: bit_matrix,
                _schedule: schedule
            }
        }
    }
}