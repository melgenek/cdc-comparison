use byteorder::{BigEndian, ReadBytesExt};
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
