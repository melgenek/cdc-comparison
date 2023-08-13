use crate::util::unsigned_integer::UnsignedInteger;

pub type MaskBuilder<T> = fn(u32) -> T;

pub fn create_simple_mask<T: UnsignedInteger>(bits_count: u32) -> T {
    (T::one() << bits_count as usize) - T::one()
}
