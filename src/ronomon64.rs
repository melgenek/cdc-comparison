use crate::chunk_sizes::ChunkSizes;
use crate::chunker::Chunker;
use crate::util::logarithm2;
use std::io::Cursor;

use aes::cipher::{generic_array::GenericArray, KeyIvInit, StreamCipher};
use byteorder::{BigEndian, ReadBytesExt};
use markdown_table::{Heading, MarkdownTable};

pub const MINIMUM_MIN: usize = 64;
pub const AVERAGE_MIN: usize = 64;
pub const AVERAGE_MAX: usize = 268435456;

pub struct Ronomon64Cdc {
    table: [u64; 256],
    mask_s: u64,
    mask_l: u64,
}

type Aes256Ctr64BE = ctr::Ctr64BE<aes::Aes256>;

fn generate_table() -> [u64; 256] {
    let max_value: u64 = 1u64 << 63;
    let mut table = [0u8; 2048];
    let key = GenericArray::from([0u8; 32]);
    let nonce = GenericArray::from([0u8; 16]);
    let mut cipher = Aes256Ctr64BE::new(&key, &nonce);
    cipher.apply_keystream(&mut table);
    let mut rdr = Cursor::new(&table[..]);
    let mut result = [0u64; 256];
    for i in 0..256 {
        let mut num: u64 = rdr.read_u64::<BigEndian>().unwrap();
        num %= max_value;
        assert!(num >= 0 && num < max_value);
        result[i] = num;
    }
    result
}

///
/// Returns two raised to the `bits` power, minus one. In other words, a bit
/// mask with that many least-significant bits set to 1.
///
fn mask(bits: u32) -> u64 {
    assert!(bits >= 1);
    assert!(bits <= 63);
    2u64.pow(bits) - 1
}

impl Ronomon64Cdc {
    pub fn new(chunk_sizes: ChunkSizes, normalization_level: u32) -> Self {
        assert!(chunk_sizes.min_size() >= MINIMUM_MIN);
        assert!(chunk_sizes.avg_size() >= AVERAGE_MIN && chunk_sizes.avg_size() <= AVERAGE_MAX);
        let bits = logarithm2(chunk_sizes.avg_size() as u32);
        Self {
            table: generate_table(),
            mask_s: mask(bits + normalization_level),
            mask_l: mask(bits - normalization_level),
        }
    }
}

///
/// Integer division that rounds up instead of down.
///
fn ceil_div(x: usize, y: usize) -> usize {
    (x + y - 1) / y
}

///
/// Find the middle of the desired chunk size, or what the FastCDC paper refers
/// to as the "normal size".
///
fn center_size(average: usize, minimum: usize, source_size: usize) -> usize {
    let mut offset: usize = minimum + ceil_div(minimum, 2);
    if offset > average {
        offset = average;
    }
    let size: usize = average - offset;
    if size > source_size {
        source_size
    } else {
        size
    }
}

impl Chunker for Ronomon64Cdc {
    fn find_split_point(&self, buf: &[u8], chunk_sizes: &ChunkSizes) -> usize {
        let buf_length = buf.len();
        let center = center_size(chunk_sizes.avg_size(), chunk_sizes.min_size(), buf_length);
        let mut index = chunk_sizes.min_size();

        let mut hash: u64 = 0;
        while index < center {
            hash = (hash >> 1) + (self.table[buf[index] as usize]);
            if (hash & self.mask_s) == 0 {
                return index;
            }
            index += 1;
        }

        while index < buf_length {
            hash = (hash >> 1) + (self.table[buf[index] as usize]);
            if (hash & self.mask_l) == 0 {
                return index;
            }
            index += 1;
        }

        index
    }
}
