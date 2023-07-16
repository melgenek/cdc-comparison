use crate::chunk_stream::{ChunkSizes, SplitPointFinder};

pub struct FixedSize;

impl FixedSize {
    pub fn new() -> Self {
        Self
    }
}

impl SplitPointFinder for FixedSize {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        chunk_sizes.avg_size().min(buf.len())
    }
}
