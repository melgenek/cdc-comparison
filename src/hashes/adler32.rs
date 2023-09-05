use crate::hashes::{RollingHash, RollingHashBuilder};
use adler32::RollingAdler32;

pub struct Adler32Builder {
    window_size: usize,
}

impl Adler32Builder {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

impl RollingHashBuilder<u32> for Adler32Builder {
    type RH<'a> = Adler32<'a>;

    fn prepare_bytes_count(&self) -> usize {
        self.window_size
    }

    fn new_hash(&self, buffer: &[u8]) -> Self::RH<'_> {
        Adler32::new(self, buffer)
    }
}

pub struct Adler32<'a> {
    builder: &'a Adler32Builder,
    digest: RollingAdler32,
    window: Vec<u8>,
    window_idx: usize,
}

impl<'a> Adler32<'a> {
    fn new(builder: &'a Adler32Builder, buffer: &[u8]) -> Self {
        let mut hash =
            Self { builder, digest: RollingAdler32::new(), window: vec![0; builder.window_size], window_idx: 0 };

        for new_byte in buffer {
            let _ = hash.replace_and_return_oldest_window_byte(*new_byte);
            hash.digest.update(*new_byte);
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

impl<'a> RollingHash<'a, u32> for Adler32<'a> {
    fn roll(&mut self, new_byte: u8) {
        let old_byte = self.replace_and_return_oldest_window_byte(new_byte);
        self.digest.remove(self.builder.window_size, old_byte);
        self.digest.update(new_byte);
    }

    fn digest(&self) -> u32 {
        self.digest.hash()
    }
}
