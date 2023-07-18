use crate::chunk_sizes::ChunkSizes;

pub trait Chunker {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize;
}
