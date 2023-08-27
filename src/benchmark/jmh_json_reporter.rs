use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::{limit_precision, size_to_str, KB};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JmhMetric {
    score: f64,
    score_unit: &'static str,
    score_error: f64,
    raw_data: Vec<Vec<f64>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JmhBenchmark {
    benchmark: String,
    mode: &'static str,
    primary_metric: JmhMetric,
    secondary_metrics: HashMap<&'static str, JmhMetric>,
}

pub fn write_results_jmh_json(
    index: usize,
    output_dir: &Path,
    avg_size: usize,
    results: &HashMap<ChunkSizes, Vec<AlgorithmResult>>,
) -> std::io::Result<()> {
    for (chunk_sizes, algorithm_results) in results {
        write_result_jmh_json(index, output_dir, avg_size, chunk_sizes, algorithm_results)?;
    }
    Ok(())
}

fn write_result_jmh_json(
    index: usize,
    output_dir: &Path,
    avg_size: usize,
    chunk_sizes: &ChunkSizes,
    results: &Vec<AlgorithmResult>,
) -> std::io::Result<()> {
    let output_dir = output_dir.join("json");
    fs::create_dir_all(&output_dir)?;
    let f = fs::OpenOptions::new().write(true).truncate(true).create(true).open(output_dir.join(format!(
        "{}_{}_{}.json",
        index,
        size_to_str(avg_size),
        chunk_sizes_to_relative_str(chunk_sizes)
    )))?;

    let jmh_benchmarks: Vec<JmhBenchmark> = results
        .into_iter()
        .map(move |result| JmhBenchmark {
            benchmark: format!("dedup.{}", result.name()),
            mode: "dedup",
            primary_metric: JmhMetric {
                score: limit_precision(result.dedup_ratio()),
                score_unit: "%",
                score_error: 0.0,
                raw_data: vec![vec![limit_precision(result.dedup_ratio())]],
            },
            secondary_metrics: HashMap::from([
                (
                    "chunk_count",
                    JmhMetric {
                        score: result.chunk_count() as f64,
                        score_unit: "count",
                        score_error: 0.0,
                        raw_data: vec![vec![result.chunk_count() as f64]],
                    },
                ),
                (
                    "chunk_size",
                    JmhMetric {
                        score: limit_precision(result.chunk_size_avg() / KB as f64),
                        score_unit: "KB",
                        score_error: limit_precision(result.chunk_size_std() / KB as f64),
                        raw_data: vec![vec![
                            limit_precision(result.min_not_last_chunk_size() / KB as f64),
                            limit_precision(result.max_chunk_size() / KB as f64),
                        ]],
                    },
                ),
                (
                    "duration",
                    JmhMetric {
                        score: limit_precision(result.duration_seconds() as f64),
                        score_unit: "second",
                        score_error: 0.0,
                        raw_data: vec![vec![limit_precision(result.duration_seconds() as f64)]],
                    },
                ),
            ]),
        })
        .collect();

    serde_json::to_writer(f, &jmh_benchmarks)?;
    Ok(())
}

fn chunk_sizes_to_relative_str(chunk_sizes: &ChunkSizes) -> String {
    format!(
        "{}*avg_avg_{}*avg",
        (chunk_sizes.min_size() as f64 / chunk_sizes.avg_size() as f64),
        (chunk_sizes.max_size() as f64 / chunk_sizes.avg_size() as f64)
    )
}
