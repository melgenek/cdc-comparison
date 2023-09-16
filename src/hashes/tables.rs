use crate::util::unsigned_integer::UnsignedInteger;
use byteorder::{BigEndian, ReadBytesExt};
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use ring::digest::{Context, SHA256};
use std::io::Cursor;

pub fn sha256_u128_table() -> [u128; 256] {
    let mut result = [0u128; 256];
    for i in 0..=255 {
        let mut seed = [0u8; 64];
        seed.fill(i);
        let mut hash = Context::new(&SHA256);
        hash.update(&seed);
        let digest = hash.finish();
        let mut rdr = Cursor::new(digest.as_ref());
        result[i as usize] = rdr.read_u128::<BigEndian>().unwrap();
    }
    result
}

pub fn sha256_u64_table() -> [u64; 256] {
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

pub fn sha256_u32_table() -> [u32; 256] {
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

// http://www.serve.net/buz/Notes.1st.year/HTML/C6/rand.012.html
pub fn buz_table<T: UnsignedInteger>() -> [T; 256] {
    let mut rng = ChaCha20Rng::seed_from_u64(1);
    let mut result = [T::zero(); 256];
    let mut indices = (0..=255).collect::<Vec<usize>>();
    for _ in 0..=T::signed_bits_count() {
        indices.shuffle(&mut rng);
        for j in 0..=127 {
            result[indices[j]] = (result[indices[j]] << 1) | T::one();
        }
        for j in 128..=255 {
            result[indices[j]] = result[indices[j]] << 1;
        }
    }
    result
}
