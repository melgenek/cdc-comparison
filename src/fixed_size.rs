use crate::chunk_sizes::ChunkSizes;
use crate::chunker::Chunker;

pub struct Fixed;

/// Splits inputs into chunks of a fixed average size.
impl Fixed {
    pub fn new() -> Self {
        Self
    }
}

impl Chunker for Fixed {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        chunk_sizes.avg_size().min(buf.len())
    }
}
