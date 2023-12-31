use crate::hashes::{RollingHash, RollingHashBuilder};
use crate::util::unsigned_integer::UnsignedInteger;

pub struct BuzHashBuilder<T: UnsignedInteger> {
    table: [T; 256],
    window_size: usize,
}

impl<T: UnsignedInteger> BuzHashBuilder<T> {
    pub fn new(table: [T; 256], window_size: usize) -> Self {
        assert!((window_size as u32) < u32::MAX);
        Self { table, window_size }
    }
}

impl<T: UnsignedInteger> RollingHashBuilder<T> for BuzHashBuilder<T> {
    type RH<'a> = BuzHash<'a, T> where T:'a;

    fn prepare_bytes_count(&self) -> usize {
        self.window_size
    }

    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_> {
        BuzHash::new(self, buffer)
    }
}

pub struct BuzHash<'a, T: UnsignedInteger> {
    builder: &'a BuzHashBuilder<T>,
    digest: T,
    window: Vec<u8>,
    window_idx: usize,
}

impl<'a, T: UnsignedInteger> BuzHash<'a, T> {
    fn new(builder: &'a BuzHashBuilder<T>, buffer: &[u8]) -> Self {
        let mut hash = Self { builder, digest: T::zero(), window: vec![0; builder.window_size], window_idx: 0 };

        for new_byte in buffer {
            let _ = hash.replace_and_return_oldest_window_byte(*new_byte);
            hash.digest = hash.digest.rotate_left(1) ^ hash.builder.table[*new_byte as usize];
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

impl<'a, T: UnsignedInteger> RollingHash<'a, T> for BuzHash<'a, T> {
    fn roll(&mut self, new_byte: u8) {
        let old_byte = self.replace_and_return_oldest_window_byte(new_byte);
        self.digest = self.digest.rotate_left(1)
            ^ self.builder.table[old_byte as usize].rotate_left(self.builder.window_size as u32)
            ^ self.builder.table[new_byte as usize];
    }

    fn digest(&self) -> T {
        self.digest
    }
}
