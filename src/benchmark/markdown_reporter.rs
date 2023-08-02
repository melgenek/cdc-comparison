use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::benchmark::ChunkerName;
use crate::util::{size_to_str, size_to_str_f64};
use markdown_table::{Heading, MarkdownTable};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

pub fn write_results_markdown(
    output_dir: &Path,
    avg_size: usize,
    results: &HashMap<ChunkerName, Vec<AlgorithmResult>>,
) -> std::io::Result<()> {
    let output_dir = output_dir.join("markdown");
    fs::create_dir_all(&output_dir)?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_dir.join(size_to_str(avg_size)).with_extension("md"))?;
    f.write_all(b"### Deduplication ratio % (the more, the better):\n\n")?;
    f.write_all(
        convert_results_to_markdown(
            results,
            |value| format!("{:.3}%", value.dedup_ratio()),
            |r1, r2| {
                let dedup_ratio1 = r1.dedup_ratio();
                let dedup_ratio2 = r2.dedup_ratio();
                if dedup_ratio1 == 0f64 && dedup_ratio2 == 0f64 {
                    Ordering::Greater
                } else {
                    dedup_ratio2.total_cmp(&dedup_ratio1)
                }
            },
        )
        .as_bytes(),
    )?;
    f.write_all(b"\n### Chunk count:\n\n")?;
    f.write_all(
        convert_results_to_markdown(
            results,
            |result| format!("{}", result.chunk_count()),
            |r1, r2| r1.chunk_count().cmp(&r2.chunk_count()),
        )
        .as_bytes(),
    )?;
    f.write_all(b"\n### Chunk sizes:\n\n")?;
    f.write_all(
        convert_results_to_markdown(
            results,
            |result| {
                format!("{}±{}", size_to_str_f64(result.chunk_size_avg()), size_to_str_f64(result.chunk_size_std()))
            },
            |r1, r2| {
                let r1_diff = (r1.chunk_size_avg() - avg_size as f64).abs();
                let r2_diff = (r2.chunk_size_avg() - avg_size as f64).abs();
                r1_diff.total_cmp(&r2_diff)
            },
        )
        .as_bytes(),
    )?;
    f.write_all(b"\n### Duration seconds:\n\n")?;
    f.write_all(
        convert_results_to_markdown(
            results,
            |result| format!("{:.2}", result.duration_seconds()),
            |r1, r2| r1.duration_seconds().total_cmp(&r2.duration_seconds()),
        )
        .as_bytes(),
    )?;
    Ok(())
}

pub fn convert_results_to_markdown<F, C>(
    results: &HashMap<ChunkerName, Vec<AlgorithmResult>>,
    result_to_string: F,
    comparator: C,
) -> String
where
    F: Fn(&AlgorithmResult) -> String,
    C: Fn(&AlgorithmResult, &AlgorithmResult) -> Ordering,
{
    let column_names = results
        .values()
        .next()
        .map_or_else(|| Vec::new(), |results| results.iter().map(|result| result.chunk_sizes()).collect());
    let mut headings: Vec<Heading> = column_names.iter().map(|name| Heading::new(name.to_string(), None)).collect();
    headings.insert(0, Heading::new("names/chunk sizes".to_string(), None));

    let values: Vec<Vec<String>> = results
        .into_iter()
        .map(|(chunker_name, values)| {
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
            row.insert(0, chunker_name.to_string());
            row
        })
        .collect();
    let mut table = MarkdownTable::new(values);
    table.with_headings(headings);
    format!("{}", table.as_markdown().unwrap())
}
