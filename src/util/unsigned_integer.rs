use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Sub};

pub trait UnsignedInteger:
    BitXor<Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + PartialEq
    + Sub<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + Sized
    + Copy
{
    fn zero() -> Self;
    fn one() -> Self;
    fn bits_count() -> usize;
}

impl UnsignedInteger for u32 {
    fn zero() -> u32 {
        0
    }

    fn one() -> Self {
        1
    }

    fn bits_count() -> usize {
        32
    }
}

impl UnsignedInteger for u64 {
    fn zero() -> u64 {
        0
    }

    fn one() -> Self {
        1
    }

    fn bits_count() -> usize {
        64
    }
}
