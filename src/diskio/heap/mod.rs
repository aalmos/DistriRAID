use std::result;
use super::chunk::Chunk;
use super::device::{StorageDevice, DeviceError};
use super::device;

pub struct Heap<'a> {
    device: &'a mut StorageDevice,
    top_chunk: Chunk,
}

pub enum HeapError {
    DeviceError(DeviceError),
    NotEnoughSpace
}

pub type HeapResult<T> = result::Result<T, HeapError>;
//
//trait ToHeapResult<T> {
//    fn to_heap_result(self) -> HeapResult<T>;
//}
//
//impl ToHeapResult<Chunk> for ChunkResult {
//    fn to_heap_result(self) -> HeapResult<Chunk> {
//        self.or_else(|error| {
//            match error {
//                ChunkOperationError::DeviceError(inner) => {
//                    Err(HeapError::DeviceError(inner))
//                },
//                _ => {
//                    Err(HeapError::NotEnoughSpace)
//                }
//            }
//        })
//    }
//}

impl<'a> Heap<'a> {
//    fn _next_free_chunk(&self, chunk: &Chunk) -> HeapResult<Chunk> {
//        chunk.next_free(self.device)
//            .or_else(|error| {
//                match error {
//                    ChunkOperationError::DeviceError(inner) => {
//                        Err(HeapError::DeviceError(inner))
//                    },
//                    _ => {
//                        Err(HeapError::NotEnoughSpace)
//                    }
//                }
//            })
//    }

//    fn _next_free_chunk_in_order(&self, chunk: &Chunk) -> HeapResult

    pub fn allocate(&mut self, size: usize) -> device::Result<u64> {
//        let chunk_found = self.top_chunk.as_freelist(self.device).find(|result| {
//            match result {
//                Ok(chunk) => chunk.data_size() >= size,
//                Err(_) => true
//            }
//        });
//
//        let chunk = match chunk_found {
//            Some(Ok(chunk)) => chunk,
//            Some(error) => return error,
//            None => return Err(device::DeviceError::NotEnoughSpace)
//        };

        Ok(1)



        //        let chunk_found = self.top_chunk.iter_freelist(self.device).find(|result| {
//            match result {
//                Ok(chunk) => chunk.data_size() >= size,
//                Err(_) => true
//            }
//        });
//
//
//        let prev = chunk.iter_neighbours(self.device).rev().nth(0);
//        let next = chunk.iter_neighbours(self.device).nth(0);
//
//        let chunk = chunk_on_device?;
//
//
//        for result in self.top_chunk.iter_freelist(self.device) {
//            match result {
//                Ok(freelist)
//            }
//        }
//        let mut current = self.top_chunk.clone();
//
//        while current.data_size() < size {
//            current = current.next_free(self.device).to_heap_result()?;
//        }
//
//        if current.data_size() - size == 0 {
//            return Ok(current.offset());
//        }
//
//        let prev = current.prev_free_in_order(self.device);
//        let next = current.next_free_in_order(self.device);
//
//        match (prev, next)


        //current.prev_free_in_order()

    }
}
