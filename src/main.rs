use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::casync::Casync;
use crate::chunk_sizes::ChunkSizes;
use markdown_table::{Heading, MarkdownTable};

use crate::chunk_stream::{Chunk, ChunkStream};
use crate::chunker::Chunker;
use crate::fast_cdc2016::FastCdc2016;
use crate::fast_cdc2020::FastCdc2020;
use crate::fixed_size::FixedSize;
use crate::google_stadia_cdc::GoogleStadiaCdc;
use crate::read_dir::MultiFileRead;
use crate::restic::chunker::ResticCdc;
use crate::restic::polynomial::Pol;
use crate::ronomon::RonomonCdc;
use crate::ronomon64::Ronomon64Cdc;
use crate::util::{
    read_files_in_dir_sorted_by_name, read_files_in_dir_sorted_by_size_desc, sha256, size_to_str_f64, KB, MB,
};

mod buzhash_tmp;
mod casync;
mod chunk_sizes;
mod chunk_stream;
mod chunker;
mod fast_cdc2016;
mod fast_cdc2020;
mod fixed_size;
mod google_stadia_cdc;
mod read_dir;
mod restic;
mod ronomon;
mod ronomon64;
mod util;

type ChunkerBuilder = Box<dyn Fn(ChunkSizes) -> Box<dyn Chunker>>;
type NamedChunker = (String, ChunkerBuilder);

fn main() -> std::io::Result<()> {
    let chunk_sizes = vec![
        ChunkSizes::new(64 * KB, 32 * KB, 256 * KB), // unusual avg*2>avg;avg*8 https://discuss.ipfs.tech/t/draft-common-bytes-standard-for-data-deduplication/6813/13  --- fails with stadia cdc
        ChunkSizes::new(32 * KB, 64 * KB, 128 * KB), // simple avg/2;avg;avg*2
        ChunkSizes::new(32 * KB, 64 * KB, 192 * KB), // ronomon good dedup avg/2;avg;3*avg https://github.com/ronomon/deduplication/issues/8#issue-810116157
        ChunkSizes::new(32 * KB, 64 * KB, 256 * KB), // ronomon good dedup avg/2;avg;3*avg(4avg) https://github.com/ronomon/deduplication/issues/8#issue-810116157
        ChunkSizes::new(16 * KB, 64 * KB, 256 * KB), // default casync avg/4;avg;avg*4 https://github.com/systemd/casync/blob/main/src/cachunker.h#L16-L20
        ChunkSizes::new(32 * KB, 64 * KB, 96 * KB), // RC4 avg/2;avg;<=avg*2 https://github.com/dbaarda/rollsum-chunking/blob/master/RESULTS.rst#summary
        ChunkSizes::new(512 * KB, 2 * MB, 8 * MB), // default duplicacy avg/4;avg;avg*4 https://github.com/gilbertchen/duplicacy/blob/master/duplicacy_paper.pdf
        ChunkSizes::new(512 * KB, 1 * MB, 8 * MB), // default restic avg/2;avg;avg*8 https://github.com/restic/chunker/blob/master/chunker.go#L15-L18
        // similar distributions, but bigger avg chunks
        ChunkSizes::new(4 * MB, 2 * MB, 16 * MB), // unusual avg*2;avg;avg*8 --- fails with stadia cdc
        ChunkSizes::new(1 * MB, 2 * MB, 16 * MB), // restic avg/2;avg;avg*8
        ChunkSizes::new(2 * MB, 4 * MB, 7 * MB),  // RC4 avg/2;avg;<=avg*2
        ChunkSizes::new(2 * MB, 4 * MB, 8 * MB),  // simple avg/2;avg;avg*2
        ChunkSizes::new(1 * MB, 4 * MB, 16 * MB), // casync/duplicacy avg/4;avg;avg*4
        ChunkSizes::new(2 * MB, 4 * MB, 12 * MB), // ronomon good dedup avg/2;avg;3*avg
        ChunkSizes::new(2 * MB, 4 * MB, 16 * MB), // ronomon good dedup avg/2;avg;>3*avg(4avg)
                                                  // ChunkSizes::new(4 * MB, 8 * MB, 16 * MB), // simple avg/2;avg;avg*2
                                                  // ChunkSizes::new(4 * MB, 8 * MB, 10 * MB), // RC4 avg/2;avg;<=avg*2
                                                  // ChunkSizes::new(4 * MB, 8 * MB, 12 * MB), // RC4 avg/2;avg;<=avg*2
                                                  // ChunkSizes::new(4 * MB, 8 * MB, 14 * MB), // RC4 avg/2;avg;<=avg*2
                                                  // ChunkSizes::new(6 * MB, 8 * MB, 10 * MB), // random <avg;avg;<=avg*2
                                                  // ChunkSizes::new(6 * MB, 8 * MB, 12 * MB), // random <avg;avg;<=avg*2
                                                  // ChunkSizes::new(8 * MB, 8 * MB, 10 * MB), // random avg;avg;<=avg*2
                                                  // ChunkSizes::new(8 * MB, 8 * MB, 12 * MB), // random avg;avg;<=avg*2
                                                  // ChunkSizes::new(8 * MB, 8 * MB, 16 * MB), // random avg;avg;<=avg*2
    ];
    let chunkers_with_names: Vec<NamedChunker> = vec![
        ("Fixed size".to_string(), Box::new(|_| Box::new(FixedSize::new()))),
        ("FastCdc2016".to_string(), Box::new(|sizes| Box::new(FastCdc2016::new(sizes, 2)))),
        ("FastCdc2016NC1".to_string(), Box::new(|sizes| Box::new(FastCdc2016::new(sizes, 1)))),
        ("FastCdc2016".to_string(), Box::new(|sizes| Box::new(FastCdc2016::new(sizes, 2)))),
        ("FastCdc2016NC3".to_string(), Box::new(|sizes| Box::new(FastCdc2016::new(sizes, 3)))),
        ("FastCdc2020NC1".to_string(), Box::new(|sizes| Box::new(FastCdc2020::new(sizes, 1)))),
        ("FastCdc2020".to_string(), Box::new(|sizes| Box::new(FastCdc2020::new(sizes, 2)))),
        ("FastCdc2020NC3".to_string(), Box::new(|sizes| Box::new(FastCdc2020::new(sizes, 3)))),
        ("Restic".to_string(), Box::new(|sizes| Box::new(ResticCdc::new(Pol::generate_random(), sizes)))),
        ("StadiaCdc".to_string(), Box::new(|sizes| Box::new(GoogleStadiaCdc::new(sizes)))),
        ("Casync".to_string(), Box::new(|sizes| Box::new(Casync::new(sizes)))),
        ("Ronomon".to_string(), Box::new(|sizes| Box::new(RonomonCdc::new(sizes, 1)))),
        ("RonomonNC2".to_string(), Box::new(|sizes| Box::new(RonomonCdc::new(sizes, 2)))),
        ("RonomonNC3".to_string(), Box::new(|sizes| Box::new(RonomonCdc::new(sizes, 3)))),
        ("Ronomon64".to_string(), Box::new(|sizes| Box::new(Ronomon64Cdc::new(sizes, 1)))),
        ("Ronomon64NC2".to_string(), Box::new(|sizes| Box::new(Ronomon64Cdc::new(sizes, 2)))),
        ("Ronomon64NC3".to_string(), Box::new(|sizes| Box::new(Ronomon64Cdc::new(sizes, 3)))),
    ];

    let chunker_names: Vec<String> = chunkers_with_names
        .iter()
        .flat_map(|(name, _)| {
            vec![
                format!("{} (concat by name)", name),
                format!("{} (concat by size)", name),
                format!("{} (files)", name),
            ]
        })
        .collect();
    let chunkers: Vec<ChunkerBuilder> = chunkers_with_names.into_iter().map(|(_, chunker)| chunker).collect();

    let mut result: Vec<(ChunkSizes, Vec<CdcResult>)> = vec![];
    for sizes in chunk_sizes {
        eprint!("{}", sizes);
        let mut chunk_size_result: Vec<CdcResult> = vec![];
        for chunker in chunkers.iter() {
            chunk_size_result.push(run_without_file_boundaries(sizes, &chunker, &read_files_in_dir_sorted_by_name)?);
            eprint!(".");
            chunk_size_result.push(run_without_file_boundaries(
                sizes,
                &chunker,
                &read_files_in_dir_sorted_by_size_desc,
            )?);
            eprint!(".");
            chunk_size_result.push(run_with_file_boundaries(sizes, &chunker)?);
            eprint!(".");
        }
        result.push((sizes, chunk_size_result));
        write_results(&chunker_names, &result)?;
        eprintln!()
    }

    Ok(())
}

fn run_without_file_boundaries<F>(
    chunk_sizes: ChunkSizes,
    chunker_builder: &ChunkerBuilder,
    get_files: &F,
) -> std::io::Result<CdcResult>
where
    F: Fn(&'static str) -> Vec<PathBuf> + 'static,
{
    let chunker = chunker_builder(chunk_sizes);
    let mut cdc_result = CdcResult::new(false);
    let mut process_directory = |dir: &'static str| -> std::io::Result<()> {
        let source = BufReader::with_capacity(16 * MB, MultiFileRead::new(get_files(dir))?);
        for result in ChunkStream::new(source, &chunker, chunk_sizes) {
            let chunk = result?;
            cdc_result.append_chunk(chunk);
        }
        Ok(())
    };
    process_directory("extracted/postgres-15.2-extracted")?;
    process_directory("extracted/postgres-15.3-extracted")?;
    cdc_result.complete();
    Ok(cdc_result)
}

fn run_with_file_boundaries(chunk_sizes: ChunkSizes, chunker_builder: &ChunkerBuilder) -> std::io::Result<CdcResult> {
    let chunker = chunker_builder(chunk_sizes);
    let mut cdc_result = CdcResult::new(true);
    let mut process_directory = |dir: &str| -> std::io::Result<()> {
        let files = read_files_in_dir_sorted_by_name(dir);
        for file in files {
            let source = File::open(file)?;
            for result in ChunkStream::new(source, &chunker, chunk_sizes) {
                let chunk = result?;
                cdc_result.append_chunk(chunk);
            }
        }
        Ok(())
    };
    process_directory("extracted/postgres-15.2-extracted")?;
    process_directory("extracted/postgres-15.3-extracted")?;
    cdc_result.complete();
    Ok(cdc_result)
}

fn write_results(names: &[String], results: &Vec<(ChunkSizes, Vec<CdcResult>)>) -> std::io::Result<()> {
    let mut f = std::fs::OpenOptions::new().write(true).truncate(true).create(true).open("NEW_RESULTS.md")?;
    f.write_all(b"### Deduplication ratio % (the more, the better):\n\n")?;
    f.write_all(
        convert_results_to_string(
            names,
            results,
            |value| format!("{:.3}%", value.dedup_ratio()),
            |r1, r2| {
                let dedup_ratio1 = r1.dedup_ratio();
                let dedup_ratio2 = r2.dedup_ratio();
                if dedup_ratio1 == 0f64 && dedup_ratio2 == 0f64 {
                    Ordering::Greater
                } else {
                    r1.considers_file_boundaries
                        .cmp(&r2.considers_file_boundaries)
                        .then(dedup_ratio2.total_cmp(&dedup_ratio1))
                }
            },
        )
        .as_bytes(),
    )?;
    f.write_all(b"\n### Chunk count:\n\n")?;
    f.write_all(
        convert_results_to_string(
            names,
            results,
            |result| format!("{}", result.chunk_count()),
            |r1, r2| r1.chunk_count().cmp(&r2.chunk_count()),
        )
        .as_bytes(),
    )?;
    f.write_all(b"\n### Chunk sizes:\n\n")?;
    f.write_all(
        convert_results_to_string(
            names,
            results,
            |result| {
                format!("{}±{}", size_to_str_f64(result.chunk_size_avg()), size_to_str_f64(result.chunk_size_std()))
            },
            |_, _| Ordering::Less,
        )
        .as_bytes(),
    )?;
    f.write_all(b"\n### Duration seconds:\n\n")?;
    f.write_all(
        convert_results_to_string(
            names,
            results,
            |result| format!("{:.2}", result.duration_seconds()),
            |r1, r2| r1.duration.cmp(&r2.duration),
        )
        .as_bytes(),
    )?;
    f.flush()?;
    Ok(())
}

fn convert_results_to_string<F, C>(
    names: &[String],
    results: &Vec<(ChunkSizes, Vec<CdcResult>)>,
    result_to_string: F,
    comparator: C,
) -> String
where
    F: Fn(&CdcResult) -> String,
    C: Fn(&CdcResult, &CdcResult) -> Ordering,
{
    let mut headings: Vec<Heading> = names.iter().map(|name| Heading::new(name.clone(), None)).collect();
    headings.insert(0, Heading::new("chunk sizes/names".to_string(), None));

    let values: Vec<Vec<String>> = results
        .into_iter()
        .map(|(chunk_sizes, values)| {
            let mut sorted_values = values.clone();
            sorted_values.sort_by(&comparator);
            let mut row: Vec<String> = values
                .into_iter()
                .map(|result| {
                    let str = result_to_string(result);
                    if sorted_values.get(0).map_or(false, |value| comparator(value, result) == Ordering::Equal) {
                        format!("**{}**", str)
                    } else if sorted_values.get(1).map_or(false, |value| comparator(value, result) == Ordering::Equal)
                        || sorted_values.get(2).map_or(false, |value| comparator(value, result) == Ordering::Equal)
                    {
                        format!("*{}*", str)
                    } else {
                        str
                    }
                })
                .collect();
            row.insert(0, chunk_sizes.to_string());
            row
        })
        .collect();
    let mut table = MarkdownTable::new(values);
    table.with_headings(headings);
    format!("{}", table.as_markdown().unwrap())
}

#[derive(Debug, Clone)]
struct CdcResult {
    considers_file_boundaries: bool,
    chunks: HashMap<String, usize>,
    total_size: usize,
    chunk_count: usize,
    start: Instant,
    duration: Duration,
}

impl CdcResult {
    fn new(considers_file_boundaries: bool) -> Self {
        CdcResult {
            considers_file_boundaries,
            chunks: HashMap::new(),
            total_size: 0,
            chunk_count: 0,
            start: Instant::now(),
            duration: Duration::ZERO,
        }
    }

    fn complete(&mut self) {
        self.duration = self.start.elapsed();
    }

    fn append_chunk(&mut self, chunk: Chunk) {
        self.total_size += chunk.length;
        let sha = sha256(&chunk.data);
        self.chunks.insert(sha, chunk.length);
        self.chunk_count += 1;
    }

    fn duration_seconds(&self) -> f32 {
        self.duration.as_secs_f32()
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
