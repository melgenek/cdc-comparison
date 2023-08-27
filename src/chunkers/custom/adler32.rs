use crate::chunkers::Chunker;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::logarithm2;
use adler32::RollingAdler32;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

fn generate_mask(ones_count: u32) -> u32 {
    let mut rng = ChaCha20Rng::seed_from_u64(6543833);
    (0..ones_count).fold(0, |num, _| {
        let bit_position = rng.gen_range(0..32);
        num | (1 << bit_position)
    })
}

pub struct Adler32 {
    split_mask: u32,
    window_size: usize,
}

impl Adler32 {
    pub fn new(chunk_sizes: ChunkSizes, window_size: usize) -> Self {
        Self::new_with_mask(chunk_sizes, window_size, true)
    }

    pub fn new_with_mask(chunk_sizes: ChunkSizes, window_size: usize, simple_mask: bool) -> Self {
        assert!(chunk_sizes.avg_size() <= u32::MAX as usize);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        Self { split_mask: if simple_mask { (1 << bits) - 1 } else { generate_mask(bits) }, window_size }
    }
}

impl Chunker for Adler32 {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut digest = RollingAdler32::new();
        let mut i = chunk_sizes.min_size() - self.window_size;
        while i < chunk_sizes.min_size() {
            digest.update(buf[i]);
            i += 1;
        }

        while i < buf.len() {
            if (digest.hash() & self.split_mask) == 0 {
                break;
            }
            let new_byte = buf[i];
            let old_byte = buf[i - self.window_size];
            digest.remove(self.window_size, old_byte);
            digest.update(new_byte);
            i += 1;
        }
        i
    }
}
