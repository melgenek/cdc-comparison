use std::collections::HashMap;
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

mod benchmark_result;
mod jmh_json_reporter;
mod markdown_reporter;

pub type ChunkerName = String;
pub type ChunkerBuilder = fn(ChunkSizes) -> Box<dyn Chunker>;
pub type NamedChunker = (ChunkerName, ChunkerBuilder);
pub type GetFilesInDirectoryFunction = fn(PathBuf) -> Vec<PathBuf>;
pub type AvgSizeToSizes = fn(usize) -> Vec<ChunkSizes>;

pub fn avg_to_standard_sizes(avg_size: usize) -> Vec<ChunkSizes> {
    vec![
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
    ]
}

pub fn evaluate(
    avg_sizes: Vec<usize>,
    avg_size_to_chunk_sizes: AvgSizeToSizes,
    chunkers_with_names: Vec<NamedChunker>,
    get_files: GetFilesInDirectoryFunction,
    input_dirs: Vec<PathBuf>,
    output_dir: &Path,
) -> std::io::Result<()> {
    avg_sizes.into_par_iter().enumerate().try_for_each(|(index, avg_size)| {
        run_size(
            index,
            avg_size,
            &avg_size_to_chunk_sizes,
            chunkers_with_names.clone(),
            &get_files,
            input_dirs.clone(),
            output_dir,
        )
    })
}

fn run_size(
    index: usize,
    avg_size: usize,
    avg_size_to_chunk_sizes: &AvgSizeToSizes,
    chunkers_with_names: Vec<NamedChunker>,
    get_files: &GetFilesInDirectoryFunction,
    input_dirs: Vec<PathBuf>,
    output_dir: &Path,
) -> std::io::Result<()> {
    let mut results_by_algorithm: HashMap<ChunkerName, Vec<AlgorithmResult>> = HashMap::new();
    let mut results_by_sizes: HashMap<ChunkSizes, Vec<AlgorithmResult>> = HashMap::new();
    for chunker in chunkers_with_names {
        let chunk_sizes = avg_size_to_chunk_sizes(avg_size);
        let size_results: Vec<std::io::Result<AlgorithmResult>> = chunk_sizes
            .par_iter()
            .map(|sizes| run_without_file_boundaries(input_dirs.clone(), *sizes, &chunker, &get_files))
            .collect();
        let size_results = size_results.into_iter().collect::<std::io::Result<Vec<AlgorithmResult>>>()?;
        for algorithm_result in &size_results {
            results_by_sizes.entry(*algorithm_result.chunk_sizes()).or_default().push(algorithm_result.clone());
        }
        results_by_algorithm.insert(chunker.0.clone(), size_results);
        write_results_markdown(output_dir, avg_size, &results_by_algorithm)?;
        write_results_jmh_json(index, output_dir, avg_size, &results_by_sizes)?;
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
    eprintln!("{} {}", name, chunk_sizes);
    let chunker = chunker_builder(chunk_sizes);
    let mut cdc_result = AlgorithmResult::new(name.clone(), chunk_sizes, input_dirs.len());
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
