use crate::hashes::{RollingHash, RollingHashBuilder};
use crate::util::unsigned_integer::UnsignedInteger;

pub struct GearHashBuilder<T: UnsignedInteger> {
    table: [T; 256],
}

impl<T: UnsignedInteger> GearHashBuilder<T> {
    pub fn new(table: [T; 256]) -> Self {
        Self { table }
    }
}

impl<T: UnsignedInteger> RollingHashBuilder<T> for GearHashBuilder<T> {
    type RH<'a> = GearHash<'a, T> where T:'a;

    fn prepare_bytes_count(&self) -> usize {
        1
    }

    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_> {
        GearHash::new(self, buffer)
    }
}

pub struct GearHash<'a, T: UnsignedInteger> {
    builder: &'a GearHashBuilder<T>,
    digest: T,
}

impl<'a, T: UnsignedInteger> GearHash<'a, T> {
    fn new(builder: &'a GearHashBuilder<T>, buffer: &[u8]) -> Self {
        let mut hash = Self { builder, digest: T::zero() };
        for new_byte in buffer {
            hash.roll(*new_byte);
        }
        hash
    }
}

impl<'a, T: UnsignedInteger> RollingHash<'a, T> for GearHash<'a, T> {
    fn roll(&mut self, new_byte: u8) {
        self.digest = (self.digest << 1).wrapping_add(&self.builder.table[new_byte as usize]);
    }

    fn digest(&self) -> T {
        self.digest
    }
}
