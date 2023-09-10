use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use rayon::prelude::*;

use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::benchmark::json_reporter::{merge_results_dir, prepare_json_dir, write_result_json};
use crate::chunkers::Chunker;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::chunk_stream::ChunkStream;
use crate::util::multi_file_dir::MultiFileRead;
use crate::util::{read_files_in_dir_sorted_by_name, sha256_file, MB};

mod benchmark_result;
mod json_reporter;

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
    prepare_json_dir(output_dir)?;
    let chunk_sizes_and_chunkers = chunkers_with_names.into_iter().flat_map(|chunker| {
        let chunk_sizes = avg_sizes.iter().flat_map(|avg_size| avg_size_to_chunk_sizes(*avg_size));
        std::iter::repeat(chunker).zip(chunk_sizes)
    });
    chunk_sizes_and_chunkers.par_bridge().try_for_each(|(chunker, chunk_sizes)| {
        let result = run_without_file_boundaries(input_dirs.clone(), chunk_sizes, &chunker, &get_files)?;
        write_result_json(output_dir, &result)?;
        Ok::<(), std::io::Error>(())
    })?;
    merge_results_dir(output_dir)?;
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
    let mut cdc_result = AlgorithmResult::new(name.clone(), chunk_sizes);
    let mut process_directory = |dir: PathBuf| -> std::io::Result<()> {
        let source = BufReader::with_capacity(16 * MB, MultiFileRead::new(get_files(dir))?);
        for result in ChunkStream::new(source, &chunker, chunk_sizes) {
            let chunk = result?;
            cdc_result.append_chunk(chunk);
        }
        cdc_result.complete_input();
        Ok(())
    };
    for dir in input_dirs {
        process_directory(dir)?;
    }
    Ok(cdc_result)
}

pub fn evaluate_full_files(input_dirs: Vec<PathBuf>, output_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(&output_dir)?;
    let mut files: HashMap<String, u64> = HashMap::new();
    let mut total_size: u64 = 0;
    for file_path in input_dirs.iter().flat_map(read_files_in_dir_sorted_by_name) {
        let file_hash = sha256_file(&file_path)?;
        let file_size = fs::metadata(file_path)?.len();
        total_size += file_size;
        files.insert(file_hash, file_size);
    }
    let dedup_ratio = (total_size - files.values().sum::<u64>()) as f64 / total_size as f64 * 100.0;
    let mut f = fs::OpenOptions::new().write(true).truncate(true).create(true).open(output_dir.join("full_size.md"))?;
    f.write_all(format!("Dirs: {:?}\n", input_dirs).as_bytes())?;
    f.write_all(format!("Dedup ratio: {}\n", dedup_ratio).as_bytes())?;
    Ok(())
}
