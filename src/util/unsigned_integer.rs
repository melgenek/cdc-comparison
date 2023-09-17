use std::fmt::{Binary, Display};
use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr, Sub};

use num_traits::WrappingAdd;

pub trait UnsignedInteger:
    BitXor<Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + PartialEq
    + Sub<Output = Self>
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + WrappingAdd
    + Binary
    + Display
    + Sized
    + Copy
    + 'static
{
    fn zero() -> Self;
    fn one() -> Self;
    fn bits_count() -> usize;
    fn signed_bits_count() -> usize {
        Self::bits_count() - 1
    }
    fn rotate_left(self, n: u32) -> Self;
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

    fn rotate_left(self, n: u32) -> Self {
        u32::rotate_left(self, n)
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

    fn rotate_left(self, n: u32) -> Self {
        u64::rotate_left(self, n)
    }
}

impl UnsignedInteger for u128 {
    fn zero() -> u128 {
        0
    }

    fn one() -> Self {
        1
    }

    fn bits_count() -> usize {
        128
    }

    fn rotate_left(self, n: u32) -> Self {
        u128::rotate_left(self, n)
    }
}
