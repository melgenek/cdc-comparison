use crate::util::logarithm2;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;

use crate::util::unsigned_integer::UnsignedInteger;

pub type MaskBuilder<T> = Box<dyn Fn(usize) -> T>;

pub fn create_simple_mask<T: UnsignedInteger>(target_size: usize) -> T {
    let bits_count = logarithm2(target_size as u32);
    (T::one() << bits_count as usize) - T::one()
}

pub fn create_spread_mask<T: UnsignedInteger>(target_size: usize) -> T {
    let bits_count = logarithm2(target_size as u32);
    let mut rng = ChaCha20Rng::seed_from_u64(6543833);
    let mut bit_indices: Vec<usize> = (0..T::bits_count()).into_iter().collect();
    bit_indices.shuffle(&mut rng);
    let shift_indices = &bit_indices[0..bits_count as usize];
    shift_indices.into_iter().fold(T::zero(), |num, idx| num | (T::one() << *idx))
}
