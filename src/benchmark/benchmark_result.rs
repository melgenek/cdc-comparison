use crate::util::chunk_sizes::ChunkSizes;
use crate::util::chunk_stream::Chunk;
use crate::util::sha256;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct AlgorithmResult {
    name: String,
    chunk_sizes: ChunkSizes,
    chunks: HashMap<String, usize>,
    total_size: usize,
    chunk_count: usize,
    start: Instant,
    duration: Duration,
    current_interval_duplicate: bool,
    current_interval_size: usize,
    interval_sizes: Vec<usize>,
}

impl AlgorithmResult {
    pub fn new(name: String, chunk_sizes: ChunkSizes) -> Self {
        AlgorithmResult {
            name,
            chunk_sizes,
            chunks: HashMap::new(),
            total_size: 0,
            chunk_count: 0,
            start: Instant::now(),
            duration: Duration::ZERO,
            current_interval_duplicate: false,
            current_interval_size: 0,
            interval_sizes: Vec::new(),
        }
    }

    fn reset_interval(&mut self, is_new_interval_duplicate: bool) {
        if self.current_interval_size != 0 {
            self.interval_sizes.push(self.current_interval_size);
            self.current_interval_size = 0;
            self.current_interval_duplicate = is_new_interval_duplicate;
        }
    }

    pub fn complete_input(&mut self) {
        self.duration = self.start.elapsed();
        if self.current_interval_size != 0 {
            self.reset_interval(false);
        }
    }

    pub fn append_chunk(&mut self, chunk: Chunk) {
        self.total_size += chunk.length;
        let sha = sha256(&chunk.data);
        let new_is_duplicate = self.chunks.insert(sha, chunk.length).is_some();

        match (self.current_interval_duplicate, new_is_duplicate) {
            (false, false) | (true, true) => {
                self.current_interval_size += chunk.length;
            }
            (false, true) | (true, false) => self.reset_interval(new_is_duplicate),
        }

        self.chunk_count += 1;
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn chunk_sizes(&self) -> &ChunkSizes {
        &self.chunk_sizes
    }

    pub fn duration_seconds(&self) -> f32 {
        self.duration.as_secs_f32()
    }

    pub fn dedup_size(&self) -> usize {
        self.chunks.values().sum()
    }

    pub fn dedup_ratio(&self) -> f64 {
        (self.total_size - self.dedup_size()) as f64 / self.total_size as f64 * 100.0
    }

    pub fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    pub fn interval_count(&self) -> usize {
        self.interval_sizes.len()
    }

    pub fn interval_size_avg(&self) -> f64 {
        let total_size: usize = self.interval_sizes.iter().sum();
        let count = self.interval_sizes.len();
        (total_size as f64) / (count as f64)
    }

    pub fn interval_size_std(&self) -> f64 {
        let avg = self.interval_size_avg();
        let variance = self
            .interval_sizes
            .iter()
            .map(|value| {
                let diff = avg - (*value as f64);
                diff * diff
            })
            .sum::<f64>()
            / self.interval_sizes.len() as f64;
        variance.sqrt()
    }

    pub fn min_interval_size(&self) -> f64 {
        self.interval_sizes.iter().min().map(|v| *v).unwrap() as f64
    }

    pub fn max_interval_size(&self) -> f64 {
        self.interval_sizes.iter().max().map(|v| *v).unwrap() as f64
    }

    pub fn chunk_size_avg(&self) -> f64 {
        (self.total_size as f64) / (self.chunk_count as f64)
    }

    pub fn chunk_size_std(&self) -> f64 {
        let avg = self.chunk_size_avg();
        let variance = self
            .chunks
            .values()
            .map(|value| {
                let diff = avg - (*value as f64);
                diff * diff
            })
            .sum::<f64>()
            / self.chunk_count as f64;
        variance.sqrt()
    }

    pub fn min_chunk_size(&self) -> f64 {
        self.chunks.values().min().map(|v| *v).unwrap() as f64
    }

    pub fn max_chunk_size(&self) -> f64 {
        self.chunks.values().max().map(|v| *v).unwrap() as f64
    }
}
