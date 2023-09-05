pub mod buzhash;
pub mod gearhash;
pub mod polynomial_hash;
pub mod right_gearhash;
pub mod tables;
pub mod adler32;

pub trait RollingHashBuilder<T> {
    type RH<'a>: RollingHash<'a, T>
    where
        Self: 'a,
        T: 'a;

    fn prepare_bytes_count(&self) -> usize;
    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_>;
}

pub trait RollingHash<'a, T> {
    fn roll(&mut self, new_byte: u8);
    fn digest(&self) -> T;
}
