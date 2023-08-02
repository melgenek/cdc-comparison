use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunk_stream::Chunk;
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
    input_count: usize,
}

impl AlgorithmResult {
    pub fn new(name: String, chunk_sizes: ChunkSizes, input_count: usize) -> Self {
        AlgorithmResult {
            name,
            chunk_sizes,
            chunks: HashMap::new(),
            total_size: 0,
            chunk_count: 0,
            start: Instant::now(),
            duration: Duration::ZERO,
            input_count,
        }
    }

    pub fn complete(&mut self) {
        self.duration = self.start.elapsed();
    }

    pub fn append_chunk(&mut self, chunk: Chunk) {
        self.total_size += chunk.length;
        let sha = sha256(&chunk.data);
        self.chunks.insert(sha, chunk.length);
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

    pub fn min_not_last_chunk_size(&self) -> f64 {
        let mut chunk_lengths: Vec<&usize> = self.chunks.values().collect();
        chunk_lengths.sort();
        let (min_chunk_size, rest) = chunk_lengths.split_first().unwrap();
        let nth_chunk_size = rest.into_iter().skip(self.input_count - 1).next().unwrap_or(min_chunk_size);

        **nth_chunk_size as f64
    }

    pub fn max_chunk_size(&self) -> f64 {
        self.chunks.values().map(|v| *v).max().unwrap() as f64
    }
}
