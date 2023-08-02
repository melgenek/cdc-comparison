use crate::benchmark::{avg_to_standard_sizes, evaluate};
use crate::chunkers::custom::gear_simple_mask::LeftGear;
use crate::util::MB;
use benchmark::NamedChunker;
use chunkers::custom::buzhash32::Buzhash32;
use chunkers::custom::buzhash32_reg::Buzhash32Reg;
use chunkers::custom::buzhash64::Buzhash64;
use chunkers::custom::buzhash64_reg::Buzhash64Reg;
use chunkers::custom::fixed_size::Fixed;
use chunkers::custom::ronomon64::Ronomon64Cdc;
use chunkers::ported::casync::Casync;
use chunkers::ported::fast_cdc2016::FastCdc2016;
use chunkers::ported::fast_cdc2020::FastCdc2020;
use chunkers::ported::google_stadia_cdc::GoogleStadiaCdc;
use chunkers::ported::restic::chunker::ResticCdc;
use chunkers::ported::restic::polynomial::Pol;
use chunkers::ported::ronomon::RonomonCdc;
use std::path::{Path, PathBuf};
use util::{read_files_in_dir_sorted_by_name, read_files_in_dir_sorted_by_size_desc, KB};

mod benchmark;
mod chunkers;
mod util;

fn main() -> std::io::Result<()> {
    evaluate_buzhash()?;
    evaluate_fast_cdc()?;
    evaluate_standard()?;
    Ok(())
}

fn evaluate_buzhash() -> std::io::Result<()> {
    let chunkers: Vec<NamedChunker> = vec![
        ("Buzhash32_32".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 32))),
        ("Buzhash32_48".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 48))),
        ("Buzhash32_64".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 64))),
        ("Buzhash32_96".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 96))),
        ("Buzhash32_128".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 128))),
        ("Buzhash32_256".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 256))),
        ("Buzhash32_512".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 512))),
        ("Buzhash32_min_chunk".to_string(), |sizes| Box::new(Buzhash32::new(sizes, sizes.min_size()))),
        ("Buzhash32Reg_32".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 32))),
        ("Buzhash32Reg_48".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 48))),
        ("Buzhash32Reg_64".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 64))),
        ("Buzhash32Reg_96".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 96))),
        ("Buzhash32Reg_128".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 128))),
        ("Buzhash32Reg_256".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 256))),
        ("Buzhash32Reg_512".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 512))),
        ("Buzhash32Reg_min_chunk".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, sizes.min_size()))),
        ("Buzhash64_32".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 32))),
        ("Buzhash64_48".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 48))),
        ("Buzhash64_64".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 64))),
        ("Buzhash64_96".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 96))),
        ("Buzhash64_128".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 128))),
        ("Buzhash64_256".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 256))),
        ("Buzhash64_512".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 512))),
        ("Buzhash64_min_chunk".to_string(), |sizes| Box::new(Buzhash64::new(sizes, sizes.min_size()))),
        ("Buzhash64Reg_32".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 32))),
        ("Buzhash64Reg_48".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 48))),
        ("Buzhash64Reg_64".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 64))),
        ("Buzhash64Reg_96".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 96))),
        ("Buzhash64Reg_128".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 128))),
        ("Buzhash64Reg_256".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 256))),
        ("Buzhash64Reg_512".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 512))),
        ("Buzhash64Reg_min_chunk".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, sizes.min_size()))),
    ];
    let input_dirs: Vec<PathBuf> = vec![
        PathBuf::from("data/extracted/postgres-15.2-extracted"),
        PathBuf::from("data/extracted/postgres-15.3-extracted"),
    ];
    evaluate(
        vec![64 * KB, 128 * KB, 256 * KB, 512 * KB, 1 * MB, 2 * MB, 4 * MB],
        avg_to_standard_sizes,
        chunkers.clone(),
        read_files_in_dir_sorted_by_name,
        input_dirs.clone(),
        Path::new("results/name_asc/buz"),
    )?;

    Ok(())
}

fn evaluate_fast_cdc() -> std::io::Result<()> {
    let chunkers: Vec<NamedChunker> = vec![
        ("FastCdc2016NC0".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 0))),
        ("FastCdc2016NC1".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 1))),
        ("FastCdc2016".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 2))),
        ("FastCdc2016NC3".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 3))),
        ("FastCdc2020NC0".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 0))),
        ("FastCdc2020NC1".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 1))),
        ("FastCdc2020".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 2))),
        ("FastCdc2020NC3".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 3))),
        ("RonomonNC0".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 0))),
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 1))),
        ("RonomonNC2".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 2))),
        ("RonomonNC3".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 3))),
        ("Ronomon64NC0".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 0))),
        ("Ronomon64".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 1))),
        ("Ronomon64NC2".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 2))),
        ("Ronomon64NC3".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 3))),
    ];
    let input_dirs: Vec<PathBuf> = vec![
        PathBuf::from("data/extracted/postgres-15.2-extracted"),
        PathBuf::from("data/extracted/postgres-15.3-extracted"),
    ];
    evaluate(
        vec![64 * KB, 128 * KB, 256 * KB, 512 * KB, 1 * MB, 2 * MB, 4 * MB],
        avg_to_standard_sizes,
        chunkers.clone(),
        read_files_in_dir_sorted_by_name,
        input_dirs.clone(),
        Path::new("results/name_asc/fast_cdc"),
    )?;

    Ok(())
}

fn evaluate_standard() -> std::io::Result<()> {
    let chunkers: Vec<NamedChunker> = vec![
        ("FixedSize".to_string(), |_| Box::new(Fixed::new())),
        ("LeftGear".to_string(), |sizes| Box::new(LeftGear::new(sizes, 2))),
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 1))),
        ("Ronomon64".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 1))),
        ("Buzhash32_64".to_string(), |sizes| Box::new(Buzhash32::new(sizes, 64))),
        ("Buzhash32Reg_64".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 64))),
        ("Buzhash64_48".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 48))),
        ("Buzhash64_64".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 64))),
        ("Buzhash64_256".to_string(), |sizes| Box::new(Buzhash64::new(sizes, 256))),
        ("Buzhash64Reg_48".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 48))),
        ("Buzhash64Reg_64".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 64))),
        ("Buzhash64Reg_256".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 256))),
        ("FastCdc2016".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 2))),
        ("FastCdc2020".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 2))),
        ("Restic".to_string(), |sizes| Box::new(ResticCdc::new(Pol::generate_random(), sizes))),
        ("StadiaCdc".to_string(), |sizes| Box::new(GoogleStadiaCdc::new(sizes))),
        ("Casync".to_string(), |sizes| Box::new(Casync::new(sizes))),
    ];
    let input_dirs: Vec<PathBuf> = vec![
        PathBuf::from("data/extracted/postgres-15.2-extracted"),
        PathBuf::from("data/extracted/postgres-15.3-extracted"),
    ];
    let avg_sizes = vec![64 * KB, 128 * KB, 256 * KB, 512 * KB, 1 * MB, 2 * MB, 4 * MB];
    evaluate(
        avg_sizes.clone(),
        avg_to_standard_sizes,
        chunkers.clone(),
        read_files_in_dir_sorted_by_name,
        input_dirs.clone(),
        Path::new("results/name_asc/standard"),
    )?;
    evaluate(
        avg_sizes,
        avg_to_standard_sizes,
        chunkers,
        read_files_in_dir_sorted_by_size_desc,
        input_dirs,
        Path::new("results/size_desc/standard"),
    )?;

    Ok(())
}
