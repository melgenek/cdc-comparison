use crate::util::chunk_sizes::ChunkSizes;

pub mod chunker_with_normalization;
pub mod custom;
pub mod fixed_size;
pub mod ported;

pub trait Chunker {
    /// Accepts a buffer and chunk sizes.
    /// The buffer is always of size [min;max).
    /// Returns the index in the buffer so that the [0;index) is the new chunk.
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize;
}
