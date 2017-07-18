use nix::unistd::{lseek64, read, write, close};
use nix::unistd::Whence;
use nix::sys::stat;

use nix::fcntl::open;
use nix::fcntl;
use nix;

pub struct NixDevice {
    fd: RawFd
}

impl NixDevice {
    pub fn open(path: &Path) -> io::Result<Box<Device>> {
        nix_result_to_io_result(open(path, fcntl::O_RDWR, stat::Mode::empty())
            .map(|fd| Box::new(NixDevice {fd: fd}) as Box<Device>))
    }
}

impl Device for NixDevice {
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> io::Result<usize> {
        nix_result_to_io_result(lseek64(self.fd, offset as i64, Whence::SeekSet)
            .and_then(|_| read(self.fd, buffer)))
    }

    fn write_at(&mut self, offset: u64, buffer: &[u8]) -> io::Result<usize> {
        nix_result_to_io_result(lseek64(self.fd, offset as i64, Whence::SeekSet)
            .and_then(|_| write(self.fd, buffer)))
    }
}

impl Drop for NixDevice {
    fn drop(&mut self) {
        close(self.fd);
    }
}