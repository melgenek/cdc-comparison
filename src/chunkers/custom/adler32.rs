use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunker::Chunker;
use crate::util::logarithm2;
use byteorder::{BigEndian, ReadBytesExt};
use ring::digest::{Context, SHA256};
use std::io::Cursor;
use adler32::RollingAdler32;

pub struct Adler32 {
    split_mask: u32,
    window_size: usize,
}

impl Adler32 {
    pub fn new(chunk_sizes: ChunkSizes, window_size: usize) -> Self {
        assert!(chunk_sizes.avg_size() <= u32::MAX as usize);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        Self { split_mask: (1 << bits) - 1, window_size }
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
            digest.remove(self.window_size,old_byte);
            digest.update(new_byte);
            i += 1;
        }
        i
    }
}
