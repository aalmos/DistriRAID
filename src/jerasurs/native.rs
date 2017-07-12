use libc::c_int;

pub type Schedule = *mut *mut c_int;
pub type BitMatrix = *mut c_int;
pub type RawBlockBuffer = *mut *mut u8;

#[link(name = "Jerasure")]
#[link(name = "gf_complete")]
extern {
    pub fn liber8tion_coding_bitmatrix(k: c_int) -> BitMatrix;

    pub fn jerasure_print_bitmatrix(
        bit_matrix: BitMatrix,
        n: c_int, m: c_int, w: c_int
    );

    pub fn jerasure_smart_bitmatrix_to_schedule(
        k: c_int, m: c_int, w: c_int,
        bit_matrix: BitMatrix
    ) -> Schedule;

    pub fn jerasure_free_schedule(schedule: Schedule);

    pub fn jerasure_schedule_encode(
        k: c_int, m: c_int, w: c_int,
        schedule: Schedule,
        data_in: RawBlockBuffer,
        coding_out: RawBlockBuffer,
        block_size: c_int,
        packet_size: c_int
    );
}