pub mod buzhash;
pub mod tables;

pub trait RollingHashBuilder<T> {
    type RH<'a>: RollingHash<'a, T>
    where
        Self: 'a;

    fn prepare_bytes_count(&self) -> usize;
    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_>;
}

pub trait RollingHash<'a, T> {
    fn roll(&mut self, new_byte: u8);
    fn digest(&self) -> T;
}
