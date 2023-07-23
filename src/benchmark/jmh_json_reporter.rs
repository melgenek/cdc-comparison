use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::util::{limit_precision, KB};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

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

fn chunk_sizes_to_bench_name(chunk_sizes: &ChunkSizes) -> String {
    format!(
        "{}*avg/avg/{}*avg",
        (chunk_sizes.min_size() as f64 / chunk_sizes.avg_size() as f64),
        (chunk_sizes.max_size() as f64 / chunk_sizes.avg_size() as f64)
    )
}

pub fn write_results_jmh_json(
    benchmark_name: &str,
    results: &Vec<(ChunkSizes, Vec<AlgorithmResult>)>,
) -> std::io::Result<()> {
    fs::create_dir_all("results/json")?;
    let f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("results/json/{}.json", benchmark_name))?;

    let jmh_benchmarks: Vec<JmhBenchmark> = results
        .into_iter()
        .flat_map(|(chunk_sizes, results)| {
            let bench_name = chunk_sizes_to_bench_name(chunk_sizes);
            results.into_iter().map(move |result| JmhBenchmark {
                benchmark: format!("{}.{}", bench_name, result.name()),
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
                                limit_precision(result.min_chunk_size() / KB as f64),
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
        })
        .collect();

    serde_json::to_writer(f, &jmh_benchmarks)?;
    Ok(())
}
