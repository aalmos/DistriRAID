use std::result;
use std::ops::Range;

pub enum DeviceError {
    NotFound,
    PermissionDenied,
    InvalidOffset,
    NotEnoughSpace,
    Other(i32)
}

pub type Result<T> = result::Result<T, DeviceError>;

pub trait StorageDevice {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize>;
    fn write_at(&mut self, offset: u64, buffer: &[u8]) -> Result<usize>;
    fn block_size(&self) -> usize;
    fn size(&self) -> usize;
    fn access_range(&self) -> Range<u64>;
}
