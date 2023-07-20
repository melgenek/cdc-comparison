use crate::util::size_to_str;
use std::fmt::{Display, Formatter};

#[derive(Copy, Clone, Debug)]
pub struct ChunkSizes {
    min_size: usize,
    avg_size: usize,
    max_size: usize,
}

impl ChunkSizes {
    pub fn new(min_size: usize, avg_size: usize, max_size: usize) -> Self {
        // assert!(min_size <= avg_size);
        // assert!(avg_size <= max_size);
        Self { min_size, avg_size, max_size }
    }

    pub fn min_size(&self) -> usize {
        self.min_size
    }
    pub fn avg_size(&self) -> usize {
        self.avg_size
    }
    pub fn max_size(&self) -> usize {
        self.max_size
    }
}

impl Display for ChunkSizes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}/{}", size_to_str(self.min_size()), size_to_str(self.avg_size()), size_to_str(self.max_size()))
    }
}
