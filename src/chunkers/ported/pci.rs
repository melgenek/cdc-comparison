// This code is ported from the https://github.com/mrd0ll4r/cdchunking-rs/blob/0e4fccb28b1a54934bbbc01f31e0399f32b29ab3/src/pci.rs

//!  A chunker implementing the Parity Check of Interval (PCI) algorithm.
//!
//!  Source:C. Zhang, D. Qi, W. Li and J. Guo, "Function of Content Defined Chunking Algorithms in
//!  Incremental Synchronization," in IEEE Access, vol. 8, pp. 5316-5330, 2020,
//!  doi: 10.1109/ACCESS.2019.2963625.
//!  PDF: https://ieeexplore.ieee.org/stamp/stamp.jsp?tp=&arnumber=8949536
use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunker::Chunker;
use probability::distribution::{Binomial, Inverse};

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

pub struct Pci {
    window_size: usize,
    threshold_hard: u32,
    threshold_simple: u32,
}

impl Pci {
    pub fn new(chunk_sizes: ChunkSizes, window_size: usize) -> Self {
        Self::new_with_nc(chunk_sizes, window_size, 0)
    }
    pub fn new_with_nc(chunk_sizes: ChunkSizes, window_size: usize, nc: u32) -> Self {
        Self {
            window_size,
            threshold_hard: threshold(chunk_sizes.avg_size() * 2usize.pow(nc), window_size.clone()),
            threshold_simple: threshold(chunk_sizes.avg_size() / 2usize.pow(nc), window_size.clone()),
        }
    }
}

impl Chunker for Pci {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut ones_count = 0;
        let mut i = chunk_sizes.min_size() - self.window_size;
        while i < chunk_sizes.min_size() {
            ones_count += buf[i].count_ones();
            i += 1;
        }

        let buf_length = buf.len();
        let center = if buf_length < chunk_sizes.avg_size() { buf_length } else { chunk_sizes.avg_size() };
        while i < center {
            if ones_count >= self.threshold_hard {
                break;
            }
            ones_count -= buf[i - self.window_size].count_ones();
            ones_count += buf[i].count_ones();
            i += 1;
        }
        while i < buf_length {
            if ones_count >= self.threshold_simple {
                break;
            }
            ones_count -= buf[i - self.window_size].count_ones();
            ones_count += buf[i].count_ones();
            i += 1;
        }
        i
    }
}
