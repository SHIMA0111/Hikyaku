use bytes::{Buf, Bytes};
use bytes_utils::SegmentedBuf;

pub struct Chunk {
    data: SegmentedBuf<Bytes>,
    size: usize,
    offset: usize,
    is_last: bool,
}

impl Chunk {
    pub fn new(data: Bytes, offset: usize, is_last: bool) -> Self {
        let size = data.len();
        let data = SegmentedBuf::from(data);
        Self {
            data,
            size,
            offset,
            is_last,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    
    pub fn is_last(&self) -> bool {
        self.is_last
    }
    
    pub fn segment_data(&self) -> &[u8] {
        self.data.chunk()
    }

    pub fn vec_data(self) -> Vec<u8> {
        self.data.into_inner().into_iter().flatten().collect()
    }

    pub fn copy_to_slice(&mut self, buf: &mut [u8]) {
        self.data.copy_to_slice(buf)
    }
    
    pub fn size(&self) -> usize {
        self.size
    }
    
    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub fn get_chunk_num(size: usize, chunk_size: usize) -> usize {
    (size as f64 / chunk_size as f64).ceil() as usize
}

pub fn chunk_range(chunk_size: usize, offset: usize) -> (usize, usize) {
    let start = offset * chunk_size;
    let end = (offset + 1) * chunk_size - 1;
    (start, end)
}
