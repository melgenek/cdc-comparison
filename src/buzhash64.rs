use crate::chunk_sizes::ChunkSizes;
use crate::chunker::Chunker;
use crate::util::logarithm2;
use byteorder::{BigEndian, ReadBytesExt};
use ring::digest::{Context, SHA256};
use std::io::Cursor;

const WINDOW_SIZE: usize = 48;

pub fn rol64(x: u64, i: usize) -> u64 {
    let i = i % 64;
    if i == 0 {
        x
    } else {
        (x << i) | (x >> (64 - i))
    }
}

fn generate_table() -> [u64; 256] {
    let mut result = [0u64; 256];
    for i in 0..=255 {
        let mut seed = [0u8; 64];
        seed.fill(i);
        let mut hash = Context::new(&SHA256);
        hash.update(&seed);
        let digest = hash.finish();
        let mut rdr = Cursor::new(digest.as_ref());
        result[i as usize] = rdr.read_u64::<BigEndian>().unwrap();
    }
    result
}

pub struct Buzhash64 {
    table: [u64; 256],
    split_mask: u64,
}

impl Buzhash64 {
    pub fn new(chunk_sizes: ChunkSizes) -> Self {
        assert!(chunk_sizes.avg_size() <= u32::MAX as usize);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        Self { table: generate_table(), split_mask: (1 << bits) - 1 }
    }
}

impl Chunker for Buzhash64 {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut hash = 0;
        let mut i = chunk_sizes.min_size() - WINDOW_SIZE;
        while i < chunk_sizes.min_size() {
            let enter = buf[i];
            hash = rol64(hash, 1) ^ self.table[enter as usize];
            i += 1;
        }

        while i < buf.len() {
            if (hash & self.split_mask) == 0 {
                break;
            }
            let new_byte = buf[i];
            let old_byte = buf[i - WINDOW_SIZE];
            hash = rol64(hash, 1) ^ rol64(self.table[old_byte as usize], WINDOW_SIZE) ^ self.table[new_byte as usize];
            i += 1;
        }
        i
    }
}
