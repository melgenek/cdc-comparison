use crate::chunkers::chunker_with_normalization::{new_normalized_chunker, ChunkerWithMask};
use crate::hashes::adler32::Adler32Builder;
use crate::hashes::buzhash::BuzHashBuilder;
use crate::hashes::gearhash::GearHashBuilder;
use crate::hashes::tables::sha256_u128_table;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::mask_builder::{create_simple_mask, create_spread_mask};
use crate::util::unsigned_integer::UnsignedInteger;

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

pub fn new_buz<T: UnsignedInteger>(
    chunk_sizes: ChunkSizes,
    table: [T; 256],
    window_size: usize,
    normalization_level: u32,
) -> ChunkerWithMask<T, BuzHashBuilder<T>, T> {
    new_normalized_chunker(
        chunk_sizes,
        BuzHashBuilder::new(table, window_size),
        Box::new(create_simple_mask),
        normalization_level,
    )
}

pub fn new_buz_spread_mask<T: UnsignedInteger>(
    chunk_sizes: ChunkSizes,
    table: [T; 256],
    window_size: usize,
    normalization_level: u32,
) -> ChunkerWithMask<T, BuzHashBuilder<T>, T> {
    new_normalized_chunker(
        chunk_sizes,
        BuzHashBuilder::new(table, window_size),
        Box::new(create_spread_mask),
        normalization_level,
    )
}

pub fn new_gear_u128(
    chunk_sizes: ChunkSizes,
    normalization_level: u32,
) -> ChunkerWithMask<u128, GearHashBuilder<u128>, u128> {
    new_normalized_chunker(
        chunk_sizes,
        GearHashBuilder::new(sha256_u128_table()),
        Box::new(create_spread_mask),
        normalization_level,
    )
}

pub fn new_adler_u32(
    chunk_sizes: ChunkSizes,
    window_size: usize,
    normalization_level: u32,
) -> ChunkerWithMask<u32, Adler32Builder, u32> {
    new_normalized_chunker(
        chunk_sizes,
        Adler32Builder::new(window_size),
        Box::new(create_simple_mask),
        normalization_level,
    )
}
