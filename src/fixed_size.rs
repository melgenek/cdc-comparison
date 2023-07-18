use crate::chunk_sizes::ChunkSizes;
use crate::chunker::Chunker;

pub struct FixedSize;

impl FixedSize {
    pub fn new() -> Self {
        Self
    }
}

impl Chunker for FixedSize {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        chunk_sizes.avg_size().min(buf.len())
    }
}
