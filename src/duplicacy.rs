use crate::chunk_sizes::ChunkSizes;
use crate::chunker::Chunker;
use crate::util::logarithm2;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use ring::digest::{Context, Digest, SHA256};
use std::io::Cursor;

pub fn rol64(x: u64, i: usize) -> u64 {
    let i = i % 64;
    if i == 0 {
        x
    } else {
        (x << i) | (x >> (64 - i))
    }
}

fn generate_table(seed: u32) -> [u64; 256] {
    fn sha256(arr: &[u8]) -> Digest {
        let mut hash = Context::new(&SHA256);
        hash.update(arr);
        hash.finish()
    }
    let mut digest = sha256(&seed.to_le_bytes());
    let mut result = [0u64; 256];

    for i in 0..64 {
        let mut rdr = Cursor::new(digest.as_ref());
        for j in 0..4 {
            result[4 * i + j] = rdr.read_u64::<LittleEndian>().unwrap();
        }
        digest = sha256(&seed.to_le_bytes());
    }

    result
}

/// https://github.com/gilbertchen/duplicacy/blob/master/src/duplicacy_chunkmaker.go#L126
pub struct Duplicacy {
    table: [u64; 256],
    split_mask: u64,
}

impl Duplicacy {
    pub fn new(chunk_sizes: ChunkSizes) -> Self {
        assert!(chunk_sizes.avg_size() <= u32::MAX as usize);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        Self { table: generate_table(8419361), split_mask: (1 << bits) - 1 }
    }
}

impl Chunker for Duplicacy {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let mut digest = 0;
        let mut i = 0;
        while i < chunk_sizes.min_size() {
            let enter = buf[i];
            digest = rol64(digest, 1) ^ self.table[enter as usize];
            i += 1;
        }

        while i < buf.len() {
            if (digest & self.split_mask) == 0 {
                break;
            }
            let new_byte = buf[i];
            let old_byte = buf[i - chunk_sizes.min_size()];
            digest = rol64(digest, 1)
                ^ rol64(self.table[old_byte as usize], chunk_sizes.min_size())
                ^ self.table[new_byte as usize];
            i += 1;
        }
        i
    }
}
