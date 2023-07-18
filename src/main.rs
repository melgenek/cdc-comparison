use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

use crate::casync::Casync;
use crate::chunk_sizes::ChunkSizes;
use markdown_table::{Heading, MarkdownTable};

use crate::chunk_stream::{Chunk, ChunkStream};
use crate::chunker::Chunker;
use crate::fast_cdc2016::FastCdc2016;
use crate::fixed_size::FixedSize;
use crate::google_stadia_cdc::GoogleStadiaCdc;
use crate::read_dir::MultiFileRead;
use crate::restic::chunker::ResticCdc;
use crate::restic::polynomial::Pol;
use crate::util::{read_files_in_dir_sorted, read_files_in_dir_split_sorted, sha256};

mod buzhash_tmp;
mod casync;
mod chunk_sizes;
mod chunk_stream;
mod chunker;
mod fast_cdc2016;
mod fixed_size;
mod google_stadia_cdc;
mod read_dir;
mod restic;
mod util;

const KB: usize = 1024;
const MB: usize = 1024 * 1024;

type ChunkerBuilder = Box<dyn Fn(ChunkSizes) -> Box<dyn Chunker>>;
type NamedChunker = (String, ChunkerBuilder);

fn main() -> std::io::Result<()> {
    let chunk_sizes = vec![
        ChunkSizes::new(16 * KB, 32 * KB, 64 * KB),
        // ChunkSizes::new(32 * KB, 64 * KB, 128 * KB),
        // ChunkSizes::new(16 * KB, 64 * KB, 256 * KB),
        // ChunkSizes::new(128 * KB, 256 * KB, 512 * KB),
        // ChunkSizes::new(512 * KB, 1 * MB, 2 * MB),
        // ChunkSizes::new(512 * KB, 1 * MB, 8 * MB),
        // ChunkSizes::new(1 * MB, 2 * MB, 4 * MB),
        // ChunkSizes::new(2 * MB, 4 * MB, 8 * MB),
        // ChunkSizes::new(2 * MB, 8 * MB, 16 * MB),
        // ChunkSizes::new(4 * MB, 8 * MB, 16 * MB),
        // ChunkSizes::new(6 * MB, 8 * MB, 10 * MB),
    ];
    let cdc_builders: Vec<NamedChunker> = vec![
        ("Fixed size".to_string(), Box::new(|_| Box::new(FixedSize::new()))),
        ("FastCdc2016".to_string(), Box::new(|sizes| Box::new(FastCdc2016::new(sizes, 2)))),
        ("Restic".to_string(), Box::new(|sizes| Box::new(ResticCdc::new(Pol::generate_random(), sizes)))),
        ("StadiaCdc".to_string(), Box::new(|sizes| Box::new(GoogleStadiaCdc::new(sizes)))),
        ("Casync".to_string(), Box::new(|sizes| Box::new(Casync::new(sizes)))),
    ];

    let mut results: BTreeMap<String, Vec<CdcResult>> = BTreeMap::new();
    for (name, builder) in cdc_builders {
        for sizes in chunk_sizes.iter() {
            println!("{}. Sizes: {:?}", name, sizes);

            let start = Instant::now();
            results
                .entry(format!("{} (concat)", name))
                .or_default()
                .push(run_without_file_boundaries(*sizes, &builder)?);
            println!("Without file boundaries is done in {}ms", start.elapsed().as_millis());

            let start = Instant::now();
            results
                .entry(format!("{} (concat split)", name))
                .or_default()
                .push(run_without_file_boundaries_split_sorted(*sizes, &builder)?);
            println!("Without file boundaries is done in {}ms", start.elapsed().as_millis());

            let start = Instant::now();
            results.entry(format!("{} (files)", name)).or_default().push(run_with_file_boundaries(*sizes, &builder)?);
            println!("With file boundaries is done in {}ms", start.elapsed().as_millis());
        }
    }

    println!("\n### Deduplication ratio % (the more, the better): ");
    display_results(&chunk_sizes, &results, |value| format!("{:.3}%", value.dedup_ratio()));

    println!("\n### Chunk count: ");
    display_results(&chunk_sizes, &results, |value| format!("{}", value.chunk_count()));

    println!("\n### Chunk sizes: ");
    display_results(&chunk_sizes, &results, |value| {
        format!("{:.3}±{:.3}", value.chunk_size_avg() / MB as f64, value.chunk_size_std() / MB as f64)
    });

    Ok(())
}

fn run_without_file_boundaries(chunk_sizes: ChunkSizes, splitter: &ChunkerBuilder) -> std::io::Result<CdcResult> {
    let mut cdc_result = CdcResult::new();
    let mut process_directory = |dir: &str| -> std::io::Result<()> {
        let source = BufReader::with_capacity(16 * MB, MultiFileRead::new(read_files_in_dir_sorted(dir))?);
        for result in ChunkStream::new(source, splitter(chunk_sizes), chunk_sizes) {
            let chunk = result?;
            cdc_result.append_chunk(chunk);
        }
        Ok(())
    };
    process_directory("extracted/postgres-15.2-extracted")?;
    process_directory("extracted/postgres-15.3-extracted")?;
    Ok(cdc_result)
}

fn run_without_file_boundaries_split_sorted(
    chunk_sizes: ChunkSizes,
    splitter: &ChunkerBuilder,
) -> std::io::Result<CdcResult> {
    let mut cdc_result = CdcResult::new();
    let mut process_directory = |dir: &str| -> std::io::Result<()> {
        let source = BufReader::with_capacity(
            16 * MB,
            MultiFileRead::new(read_files_in_dir_split_sorted(dir, chunk_sizes.max_size()))?,
        );
        for result in ChunkStream::new(source, splitter(chunk_sizes), chunk_sizes) {
            let chunk = result?;
            cdc_result.append_chunk(chunk);
        }
        Ok(())
    };
    process_directory("extracted/postgres-15.2-extracted")?;
    process_directory("extracted/postgres-15.3-extracted")?;
    Ok(cdc_result)
}

fn run_with_file_boundaries(chunk_sizes: ChunkSizes, splitter: &ChunkerBuilder) -> std::io::Result<CdcResult> {
    let mut cdc_result = CdcResult::new();
    let mut process_directory = |dir: &str| -> std::io::Result<()> {
        let files = read_files_in_dir_sorted(dir);
        for file in files {
            let source = File::open(file)?;
            for result in ChunkStream::new(source, splitter(chunk_sizes), chunk_sizes) {
                let chunk = result?;
                cdc_result.append_chunk(chunk);
            }
        }
        Ok(())
    };
    process_directory("extracted/postgres-15.2-extracted")?;
    process_directory("extracted/postgres-15.3-extracted")?;
    Ok(cdc_result)
}

fn display_results<F>(chunk_sizes: &[ChunkSizes], results: &BTreeMap<String, Vec<CdcResult>>, display: F)
where
    F: Fn(&CdcResult) -> String,
{
    let mut headings: Vec<Heading> = chunk_sizes
        .into_iter()
        .map(|sizes| {
            let label = format!(
                "{}/{}/{}",
                sizes.min_size() as f64 / MB as f64,
                sizes.avg_size() as f64 / MB as f64,
                sizes.max_size() as f64 / MB as f64
            );
            Heading::new(label, None)
        })
        .collect();
    headings.insert(0, Heading::new("names/chunk sizes".to_string(), None));

    let values: Vec<Vec<String>> = results
        .into_iter()
        .map(|(name, values)| {
            let mut row: Vec<String> = values.into_iter().map(|v| display(v)).collect();
            row.insert(0, name.to_string());
            row
        })
        .collect();

    let mut table = MarkdownTable::new(values);
    table.with_headings(headings);
    println!("{}", table.as_markdown().unwrap());
}

struct CdcResult {
    chunks: HashMap<String, usize>,
    total_size: usize,
    chunk_count: usize,
}

impl CdcResult {
    fn new() -> Self {
        CdcResult { chunks: HashMap::new(), total_size: 0, chunk_count: 0 }
    }

    fn append_chunk(&mut self, chunk: Chunk) {
        self.total_size += chunk.length;
        let sha = sha256(&chunk.data);
        self.chunks.insert(sha, chunk.length);
        self.chunk_count += 1;
    }

    fn dedup_size(&self) -> usize {
        self.chunks.values().sum()
    }

    fn dedup_ratio(&self) -> f64 {
        (self.total_size - self.dedup_size()) as f64 / self.total_size as f64 * 100.0
    }

    fn chunk_count(&self) -> usize {
        self.chunk_count
    }

    fn chunk_size_avg(&self) -> f64 {
        (self.total_size as f64) / (self.chunk_count as f64)
    }

    fn chunk_size_std(&self) -> f64 {
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
}
