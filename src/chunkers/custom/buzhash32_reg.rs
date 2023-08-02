use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunker::Chunker;
use byteorder::{BigEndian, ReadBytesExt};
use ring::digest::{Context, SHA256};
use std::io::Cursor;

fn rol32(x: u32, i: usize) -> u32 {
    let i = i % 32;
    if i == 0 {
        x
    } else {
        (x << i) | (x >> (32 - i))
    }
}

fn generate_table() -> [u32; 256] {
    let mut result = [0u32; 256];
    for i in 0..=255 {
        let mut seed = [0u8; 64];
        seed.fill(i);
        let mut hash = Context::new(&SHA256);
        hash.update(&seed);
        let digest = hash.finish();
        let mut rdr = Cursor::new(digest.as_ref());
        result[i as usize] = rdr.read_u32::<BigEndian>().unwrap();
    }
    result
}

pub struct Buzhash32Reg {
    table: [u32; 256],
    threshold: u32,
    window_size: usize,
}

impl Buzhash32Reg {
    pub fn new(chunk_sizes: ChunkSizes, window_size: usize) -> Self {
        assert!(chunk_sizes.avg_size() <= u32::MAX as usize);
        Self {
            table: generate_table(),
            threshold: u32::MAX / (chunk_sizes.avg_size() as u32 - chunk_sizes.min_size() as u32 + 1),
            window_size,
        }
    }
}

impl Chunker for Buzhash32Reg {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut digest = 0;
        let mut i = chunk_sizes.min_size() - self.window_size;
        while i < chunk_sizes.min_size() {
            let enter = buf[i];
            digest = rol32(digest, 1) ^ self.table[enter as usize];
            i += 1;
        }

        let mut rc_len = buf.len();
        let mut rc_mask: u32 = 0;
        while i < buf.len() {
            if (digest & rc_mask) == 0 {
                if digest <= self.threshold {
                    return i;
                }
                rc_len = i;
                rc_mask = u32::MAX;
                while (digest & rc_mask) > 0 {
                    rc_mask <<= 1;
                }
            }
            let new_byte = buf[i];
            let old_byte = buf[i - self.window_size];
            digest = rol32(digest, 1)
                ^ rol32(self.table[old_byte as usize], self.window_size)
                ^ self.table[new_byte as usize];
            i += 1;
        }

        if (digest & rc_mask) > 0 {
            rc_len
        } else {
            i
        }
    }
}
