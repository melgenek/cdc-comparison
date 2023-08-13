use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunker::Chunker;
use crate::hashes::RollingHash;
use crate::hashes::RollingHashBuilder;
use crate::util::logarithm2;
use crate::util::mask_builder::MaskBuilder;
use crate::util::unsigned_integer::UnsignedInteger;

pub struct ChunkerWithMask<T: UnsignedInteger, H: RollingHashBuilder<T>> {
    hash_builder: H,
    mask_low_probability: T,
    mask_high_probability: T,
}

impl<T: UnsignedInteger, H: RollingHashBuilder<T>> ChunkerWithMask<T, H> {
    pub fn new(
        chunk_sizes: ChunkSizes,
        hash_builder: H,
        mask_builder: MaskBuilder<T>,
        normalization_level: u32,
    ) -> Self {
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        Self {
            hash_builder,
            mask_low_probability: mask_builder(bits + normalization_level),
            mask_high_probability: mask_builder(bits - normalization_level),
        }
    }
}

impl<T: UnsignedInteger, H: RollingHashBuilder<T>> Chunker for ChunkerWithMask<T, H> {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let buf_length = buf.len();
        let center = if buf_length < chunk_sizes.avg_size() { buf_length } else { chunk_sizes.avg_size() };
        let mut hash = self
            .hash_builder
            .new_hash(&buf[(chunk_sizes.min_size() - self.hash_builder.prepare_bytes_count())..chunk_sizes.min_size()]);

        let mut index = chunk_sizes.min_size();
        while index < center {
            hash.roll(buf[index]);
            if (hash.digest() & self.mask_low_probability) == T::zero() {
                return index;
            }
            index += 1;
        }
        while index < buf_length {
            hash.roll(buf[index]);
            if (hash.digest() & self.mask_high_probability) == T::zero() {
                return index;
            }
            index += 1;
        }

        index
    }
}
