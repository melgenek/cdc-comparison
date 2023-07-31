use std::io::BufReader;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::benchmark::jmh_json_reporter::write_results_jmh_json;
use crate::benchmark::markdown_reporter::write_results_markdown;
use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::chunkers::chunk_stream::ChunkStream;
use crate::chunkers::chunker::Chunker;
use crate::util::multi_file_dir::MultiFileRead;
use crate::util::MB;

pub mod benchmark_result;
mod jmh_json_reporter;
pub mod markdown_reporter;

pub type ChunkerBuilder = fn(ChunkSizes) -> Box<dyn Chunker>;
pub type NamedChunker = (String, ChunkerBuilder);
pub type GetFilesInDirectoryFunction = fn(PathBuf) -> Vec<PathBuf>;

pub struct Benchmark {
    name: String,
    chunk_sizes: Vec<ChunkSizes>,
    chunkers_with_names: Vec<NamedChunker>,
    get_files: GetFilesInDirectoryFunction,
}

impl Benchmark {
    pub fn new(
        name: String,
        avg_size: usize,
        chunkers_with_names: Vec<NamedChunker>,
        get_files: GetFilesInDirectoryFunction,
    ) -> Self {
        let chunk_sizes = vec![
            ChunkSizes::new(avg_size / 2, avg_size, 2 * avg_size),
            ChunkSizes::new(avg_size / 2, avg_size, 3 * avg_size), // ronomon good dedup avg/2;avg;>=3*avg https://github.com/ronomon/deduplication/issues/8#issue-810116157
            ChunkSizes::new(avg_size / 2, avg_size, 4 * avg_size), // ronomon good dedup avg/2;avg;>=3*avg(4avg) https://github.com/ronomon/deduplication/issues/8#issue-810116157
            ChunkSizes::new(avg_size / 4, avg_size, 4 * avg_size), // casync avg/4;avg;avg*4 https://github.com/systemd/casync/blob/main/src/cachunker.h#L16-L20
            ChunkSizes::new(avg_size / 2, avg_size, (1.25 * avg_size as f64) as usize), // RC4 avg/2;avg;<=avg*2 https://github.com/dbaarda/rollsum-chunking/blob/master/RESULTS.rst#summary
            ChunkSizes::new(avg_size / 2, avg_size, (1.5 * avg_size as f64) as usize), // RC4 avg/2;avg;<=avg*2 https://github.com/dbaarda/rollsum-chunking/blob/master/RESULTS.rst#summary
            ChunkSizes::new(avg_size / 2, avg_size, (1.75 * avg_size as f64) as usize), // RC4 avg/2;avg;<=avg*2 https://github.com/dbaarda/rollsum-chunking/blob/master/RESULTS.rst#summary
            ChunkSizes::new((0.75 * avg_size as f64) as usize, avg_size, (1.5 * avg_size as f64) as usize),
            ChunkSizes::new(avg_size / 2, avg_size, 5 * avg_size),
            ChunkSizes::new(avg_size / 2, avg_size, 8 * avg_size), // restic avg/2;avg;avg*8 https://github.com/restic/chunker/blob/master/chunker.go#L15-L18
        ];
        Self { name, chunk_sizes, chunkers_with_names, get_files }
    }
}

pub fn run_benchmarks(benchmarks: Vec<Benchmark>, input_dirs: Vec<PathBuf>, output_dir: &Path) -> std::io::Result<()> {
    benchmarks.into_par_iter().try_for_each(|benchmark| run_benchmark(benchmark, input_dirs.clone(), output_dir))
}

fn run_benchmark(benchmark: Benchmark, input_dirs: Vec<PathBuf>, output_dir: &Path) -> std::io::Result<()> {
    eprintln!("Running '{}'", benchmark.name);

    let mut result: Vec<(ChunkSizes, Vec<AlgorithmResult>)> = vec![];
    for sizes in benchmark.chunk_sizes {
        eprint!("{}", sizes);
        let mut chunk_size_result: Vec<AlgorithmResult> = vec![];
        for chunker in benchmark.chunkers_with_names.iter() {
            chunk_size_result.push(run_without_file_boundaries(
                input_dirs.clone(),
                sizes,
                &chunker,
                &benchmark.get_files,
            )?);
            eprint!(".");
        }
        result.push((sizes, chunk_size_result));
        write_results_markdown(output_dir, &benchmark.name, &result)?;
        write_results_jmh_json(output_dir, &benchmark.name, &result)?;
        eprintln!()
    }

    Ok(())
}

fn run_without_file_boundaries(
    input_dirs: Vec<PathBuf>,
    chunk_sizes: ChunkSizes,
    named_chunker: &NamedChunker,
    get_files: &GetFilesInDirectoryFunction,
) -> std::io::Result<AlgorithmResult> {
    let (name, chunker_builder) = named_chunker;
    let chunker = chunker_builder(chunk_sizes);
    let mut cdc_result = AlgorithmResult::new(name.clone(), input_dirs.len());
    let mut process_directory = |dir: PathBuf| -> std::io::Result<()> {
        let source = BufReader::with_capacity(16 * MB, MultiFileRead::new(get_files(dir))?);
        for result in ChunkStream::new(source, &chunker, chunk_sizes) {
            let chunk = result?;
            cdc_result.append_chunk(chunk);
        }
        Ok(())
    };
    for dir in input_dirs {
        process_directory(dir)?;
    }
    cdc_result.complete();
    Ok(cdc_result)
}
