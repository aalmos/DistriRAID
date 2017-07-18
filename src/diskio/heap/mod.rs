use super::chunk::Chunk;
use super::device::StorageDevice;

struct Heap {
    device: StorageDevice,
    top_chunk: chunk::Chunk,
}

//impl Heap {
//    pub fn allocate(&mut self, size: usize) {
//        let mut current = self.top_chunk;
//        while current.data_size() < size {
//            match current {
//
//            }
//        }
//    }
//}

//pub trait Heap {
//
//}