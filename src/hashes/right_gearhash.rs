use crate::hashes::{RollingHash, RollingHashBuilder};
use crate::util::unsigned_integer::UnsignedInteger;

pub struct RightGearHashBuilder<T: UnsignedInteger> {
    table: [T; 256],
}

impl<T: UnsignedInteger> RightGearHashBuilder<T> {
    pub fn new(table: [T; 256]) -> Self {
        Self { table }
    }
}

impl<T: UnsignedInteger> RollingHashBuilder<T> for RightGearHashBuilder<T> {
    type RH<'a> = RightGearHash<'a, T> where T:'a;

    fn prepare_bytes_count(&self) -> usize {
        1
    }

    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_> {
        RightGearHash::new(self, buffer)
    }
}

pub struct RightGearHash<'a, T: UnsignedInteger> {
    builder: &'a RightGearHashBuilder<T>,
    digest: T,
}

impl<'a, T: UnsignedInteger> RightGearHash<'a, T> {
    fn new(builder: &'a RightGearHashBuilder<T>, buffer: &[u8]) -> Self {
        let mut hash = Self { builder, digest: T::zero() };
        for new_byte in buffer {
            hash.roll(*new_byte);
        }
        hash
    }
}

impl<'a, T: UnsignedInteger> RollingHash<'a, T> for RightGearHash<'a, T> {
    fn roll(&mut self, new_byte: u8) {
        self.digest = (self.digest >> 1) + self.builder.table[new_byte as usize];
    }

    fn digest(&self) -> T {
        self.digest
    }
}
