use crate::chunkers::Chunker;
use crate::hashes::RollingHash;
use crate::hashes::RollingHashBuilder;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::mask_builder::MaskBuilder;
use crate::util::unsigned_integer::UnsignedInteger;

type CenterFinder = fn(&ChunkSizes, usize) -> usize;
type Predicate<T, MT> = fn(T, MT) -> bool;

pub fn new_normalized_chunker<T: UnsignedInteger, H: RollingHashBuilder<T>>(
    chunk_sizes: ChunkSizes,
    hash_builder: H,
    mask_builder: MaskBuilder<T>,
    normalization_level: u32,
) -> ChunkerWithMask<T, H, T> {
    new_custom_normalized_chunker(
        chunk_sizes,
        hash_builder,
        mask_builder,
        normalization_level,
        simple_center_finder,
        simple_predicate,
    )
}

pub fn new_normalized_chunker_with_center<T: UnsignedInteger, H: RollingHashBuilder<T>>(
    chunk_sizes: ChunkSizes,
    hash_builder: H,
    mask_builder: MaskBuilder<T>,
    normalization_level: u32,
    center_finder: CenterFinder,
) -> ChunkerWithMask<T, H, T> {
    new_custom_normalized_chunker(
        chunk_sizes,
        hash_builder,
        mask_builder,
        normalization_level,
        center_finder,
        simple_predicate,
    )
}

pub fn new_normalized_chunker_with_predicate<T: UnsignedInteger, H: RollingHashBuilder<T>, MT: UnsignedInteger>(
    chunk_sizes: ChunkSizes,
    hash_builder: H,
    mask_builder: MaskBuilder<MT>,
    normalization_level: u32,
    predicate: Predicate<T, MT>,
) -> ChunkerWithMask<T, H, MT> {
    new_custom_normalized_chunker(
        chunk_sizes,
        hash_builder,
        mask_builder,
        normalization_level,
        simple_center_finder,
        predicate,
    )
}

pub fn new_custom_normalized_chunker<T: UnsignedInteger, H: RollingHashBuilder<T>, MT: UnsignedInteger>(
    chunk_sizes: ChunkSizes,
    hash_builder: H,
    mask_builder: MaskBuilder<MT>,
    normalization_level: u32,
    center_finder: CenterFinder,
    predicate: Predicate<T, MT>,
) -> ChunkerWithMask<T, H, MT> {
    ChunkerWithMask {
        hash_builder,
        center_finder,
        predicate,
        mask_low_probability: mask_builder(chunk_sizes.avg_size() << normalization_level),
        mask_high_probability: mask_builder(chunk_sizes.avg_size() >> normalization_level),
    }
}

pub fn simple_center_finder(chunk_sizes: &ChunkSizes, buf_size: usize) -> usize {
    if buf_size < chunk_sizes.avg_size() {
        buf_size
    } else {
        chunk_sizes.avg_size()
    }
}

fn simple_predicate<T: UnsignedInteger>(digest: T, mask: T) -> bool {
    (digest & mask) == T::zero()
}

pub struct ChunkerWithMask<T: UnsignedInteger, H: RollingHashBuilder<T>, MT: UnsignedInteger> {
    hash_builder: H,
    center_finder: CenterFinder,
    predicate: Predicate<T, MT>,
    mask_low_probability: MT,
    mask_high_probability: MT,
}

impl<T: UnsignedInteger, H: RollingHashBuilder<T>, MT: UnsignedInteger> Chunker for ChunkerWithMask<T, H, MT> {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let buf_length = buf.len();
        let center = (self.center_finder)(chunk_sizes, buf_length);
        let mut hash = self
            .hash_builder
            .new_hash(&buf[(chunk_sizes.min_size() - self.hash_builder.prepare_bytes_count())..chunk_sizes.min_size()]);

        let mut index = chunk_sizes.min_size();
        while index < center {
            if (self.predicate)(hash.digest(), self.mask_low_probability) {
                return index;
            }
            hash.roll(buf[index]);
            index += 1;
        }
        while index < buf_length {
            if (self.predicate)(hash.digest(), self.mask_high_probability) {
                return index;
            }
            hash.roll(buf[index]);
            index += 1;
        }

        index
    }
}
