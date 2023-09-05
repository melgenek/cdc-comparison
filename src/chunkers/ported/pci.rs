// This code is ported from the https://github.com/mrd0ll4r/cdchunking-rs/blob/0e4fccb28b1a54934bbbc01f31e0399f32b29ab3/src/pci.rs

//!  A chunker implementing the Parity Check of Interval (PCI) algorithm.
//!
//!  Source:C. Zhang, D. Qi, W. Li and J. Guo, "Function of Content Defined Chunking Algorithms in
//!  Incremental Synchronization," in IEEE Access, vol. 8, pp. 5316-5330, 2020,
//!  doi: 10.1109/ACCESS.2019.2963625.
//!  PDF: https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=8949536
use probability::distribution::{Binomial, Inverse};

use crate::chunkers::chunker_with_normalization::{
    ChunkerWithMask, new_custom_normalized_chunker, simple_center_finder,
};
use crate::hashes::{RollingHash, RollingHashBuilder};
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::mask_builder::MaskBuilder;

/// This function produces a threshold on the number of bits for binomial distribution of chunk sizes.
/// The threshold is hardcoded in the paper.
pub fn threshold(chunk_size: usize, window_size: usize) -> u32 {
    // The split point probability.
    // For example, for an average 1 KB chunk we want to have a split point once in 1024 bytes.
    let desired_probability = 1.0f64 / chunk_size as f64;
    // The binomial distribution for:
    // - probability 0.5: 0 or 1 bit
    // - number of trials: 8 (bits in byte) * window_size
    let d = Binomial::new(8 * window_size, 0.5);
    // Percent point function (also called ppf) returns the value of a random variable
    // such that its probability is less than or equal to an input.
    // The resulting value is effectively the lowest bound
    // at which the probability is no bigger than the desired probability.
    (d.inverse(1.0 - desired_probability) + 1) as u32
}

pub struct Pci;

impl Pci {
    pub fn new(
        chunk_sizes: ChunkSizes,
        window_size: usize,
        normalization_level: u32,
    ) -> ChunkerWithMask<u32, PciHashBuilder, u32> {
        let mask_builder: MaskBuilder<u32> = Box::new(move |target_size| threshold(target_size, window_size));
        new_custom_normalized_chunker(
            chunk_sizes,
            PciHashBuilder::new(window_size),
            mask_builder,
            normalization_level,
            simple_center_finder,
            |ones_count, threshold| ones_count >= threshold,
        )
    }
}

pub struct PciHashBuilder {
    window_size: usize,
}

impl PciHashBuilder {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

impl RollingHashBuilder<u32> for PciHashBuilder {
    type RH<'a> = PciHash<'a>;

    fn prepare_bytes_count(&self) -> usize {
        self.window_size
    }

    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_> {
        PciHash::new(self, buffer)
    }
}

pub struct PciHash<'a> {
    builder: &'a PciHashBuilder,
    ones_count: u32,
    window: Vec<u8>,
    window_idx: usize,
}

impl<'a> PciHash<'a> {
    fn new(builder: &'a PciHashBuilder, buffer: &[u8]) -> Self {
        let mut hash = Self { builder, ones_count: 0, window: vec![0; builder.window_size], window_idx: 0 };

        for new_byte in buffer {
            let _ = hash.replace_and_return_oldest_window_byte(*new_byte);
            hash.ones_count += new_byte.count_ones();
        }

        hash
    }

    fn replace_and_return_oldest_window_byte(&mut self, new_byte: u8) -> u8 {
        let old_byte = self.window[self.window_idx];
        self.window[self.window_idx] = new_byte;
        self.window_idx = (self.window_idx + 1) % self.builder.window_size;
        old_byte
    }
}

impl<'a> RollingHash<'a, u32> for PciHash<'a> {
    fn roll(&mut self, new_byte: u8) {
        let old_byte = self.replace_and_return_oldest_window_byte(new_byte);
        self.ones_count -= old_byte.count_ones();
        self.ones_count += new_byte.count_ones();
    }

    fn digest(&self) -> u32 {
        self.ones_count
    }
}
