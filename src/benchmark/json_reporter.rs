use itertools::Itertools;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::{read_files_in_dir_sorted_by_name, size_to_str, size_to_str_f64};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Result {
    name: String,
    chunk_sizes: String,
    dedup_ratio: String,
    duration_seconds: String,
    result_chunk_sizes: String,
    result_chunk_count: usize,
    min_chunk_size: String,
    max_chunk_size: String,
    interval_sizes: String,
    interval_count: usize,
    min_interval_size: String,
    max_interval_size: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MergedReport {
    results: Vec<Result>,
}

pub fn prepare_json_dir(output_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(&output_dir.join("runs"))
}

fn read_single_report(path: PathBuf) -> std::io::Result<Result> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let report: Result = serde_json::from_reader(reader).unwrap();
    Ok(report)
}

pub fn merge_results_dir(output_dir: &Path) -> std::io::Result<()> {
    let report_paths = read_files_in_dir_sorted_by_name(&output_dir.join("runs"));
    let results = report_paths.into_iter().map(read_single_report).collect::<std::io::Result<Vec<Result>>>()?;
    merge_buz(output_dir, results.clone())?;
    merge_all(output_dir, results.clone())?;
    Ok(())
}

fn merge_buz(output_dir: &Path, results: Vec<Result>) -> std::io::Result<()> {
    let regexp = Regex::new(r"/(.*)/").unwrap();
    let avg_size_to_results: HashMap<String, Vec<Result>> = results
        .into_iter()
        .into_grouping_map_by(|v| regexp.captures(v.chunk_sizes.as_str()).unwrap().get(1).unwrap().as_str().to_string())
        .collect();
    for (avg_size, results) in avg_size_to_results {
        let merged_report = MergedReport { results };
        let merged_file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(output_dir.join(format!("{}.json", avg_size)))?;
        serde_json::to_writer(merged_file, &merged_report)?;
    }
    Ok(())
}

fn merge_all(output_dir: &Path, results: Vec<Result>) -> std::io::Result<()> {
    let merged_report = MergedReport { results };
    let merged_file =
        fs::OpenOptions::new().write(true).truncate(true).create(true).open(output_dir.join("merged.json"))?;
    serde_json::to_writer(merged_file, &merged_report)?;
    Ok(())
}

pub fn write_result_json(output_dir: &Path, result: &AlgorithmResult) -> std::io::Result<()> {
    let f =
        fs::OpenOptions::new().write(true).truncate(true).create(true).open(output_dir.join("runs").join(format!(
            "{}_{}.json",
            result.name(),
            chunk_sizes_to_path_str(result.chunk_sizes()),
        )))?;

    let report = Result {
        name: result.name().to_string(),
        chunk_sizes: result.chunk_sizes().to_string(),
        duration_seconds: format!("{:.1}", result.duration_seconds()),
        dedup_ratio: format!("{:.3}%", result.dedup_ratio()),
        result_chunk_sizes: format!(
            "{}±{}",
            size_to_str_f64(result.chunk_size_avg()),
            size_to_str_f64(result.chunk_size_std())
        ),
        result_chunk_count: result.chunk_count(),
        min_chunk_size: size_to_str_f64(result.min_chunk_size()),
        max_chunk_size: size_to_str_f64(result.max_chunk_size()),
        interval_sizes: format!(
            "{}±{}",
            size_to_str_f64(result.interval_size_avg()),
            size_to_str_f64(result.interval_size_std())
        ),
        interval_count: result.interval_count(),
        min_interval_size: size_to_str_f64(result.min_interval_size()),
        max_interval_size: size_to_str_f64(result.max_interval_size()),
    };

    serde_json::to_writer(f, &report)?;
    Ok(())
}

fn chunk_sizes_to_path_str(chunk_sizes: &ChunkSizes) -> String {
    format!(
        "{}_{}_{}",
        size_to_str(chunk_sizes.min_size()),
        size_to_str(chunk_sizes.avg_size()),
        size_to_str(chunk_sizes.max_size())
    )
}
