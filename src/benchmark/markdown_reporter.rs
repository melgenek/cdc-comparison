use crate::benchmark::benchmark_result::AlgorithmResult;
use crate::chunkers::chunk_sizes::ChunkSizes;
use crate::util::size_to_str_f64;
use markdown_table::{Heading, MarkdownTable};
use std::cmp::Ordering;
use std::fs;
use std::io::Write;

pub fn write_results_markdown(
    benchmark_name: &str,
    results: &Vec<(ChunkSizes, Vec<AlgorithmResult>)>,
) -> std::io::Result<()> {
    fs::create_dir_all("results/markdown")?;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(format!("results/markdown/{}.md", benchmark_name))?;
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
            |_, _| Ordering::Less,
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
    results: &Vec<(ChunkSizes, Vec<AlgorithmResult>)>,
    result_to_string: F,
    comparator: C,
) -> String
where
    F: Fn(&AlgorithmResult) -> String,
    C: Fn(&AlgorithmResult, &AlgorithmResult) -> Ordering,
{
    let names: Vec<&str> =
        results.get(0).map_or_else(|| Vec::new(), |results| results.1.iter().map(|result| result.name()).collect());
    let mut headings: Vec<Heading> = names.iter().map(|name| Heading::new(name.to_string(), None)).collect();
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
