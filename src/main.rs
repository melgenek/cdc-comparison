use std::path::{Path, PathBuf};

use benchmark::NamedChunker;
use chunkers::custom::buzhash32_reg::Buzhash32Reg;
use chunkers::custom::buzhash64_reg::Buzhash64Reg;
use chunkers::fixed_size::Fixed;
use chunkers::ported::casync::Casync;
use chunkers::ported::fast_cdc2016::FastCdc2016;
use chunkers::ported::fast_cdc2020::FastCdc2020;
use chunkers::ported::google_stadia_cdc::GoogleStadiaCdc;
use chunkers::ported::ronomon::RonomonCdc;
use util::{read_files_in_dir_sorted_by_name, read_files_in_dir_sorted_by_size_desc, KB};

use crate::benchmark::{avg_to_standard_sizes, evaluate};
use crate::chunkers::ported::pci::Pci;
use crate::chunkers::ported::restic::ResticCdc;
use crate::hashes::polynomial_hash::polynomial::Pol;
use crate::util::chunk_sizes::ChunkSizes;
use crate::util::MB;

mod benchmark;
mod chunkers;
mod hashes;
mod util;

fn main() -> std::io::Result<()> {
    let chunkers: Vec<NamedChunker> = vec![("Pci_512_gen".to_string(), |sizes| Box::new(Pci::new(sizes, 512, 1)))];
    let input_dirs: Vec<PathBuf> = vec![
        PathBuf::from("data/extracted/postgres-15.2-extracted"),
        PathBuf::from("data/extracted/postgres-15.3-extracted"),
    ];
    evaluate(
        vec![64 * KB, 128 * KB, 256 * KB],
        |avg_size| vec![ChunkSizes::new(avg_size / 4, avg_size, 4 * avg_size)],
        chunkers.clone(),
        read_files_in_dir_sorted_by_name,
        input_dirs.clone(),
        Path::new("results/gen/buz"),
    )?;
    Ok(())
}

fn evaluate_buzhash() -> std::io::Result<()> {
    let chunkers: Vec<NamedChunker> = vec![
        ("Buzhash32Reg_32".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 32))),
        ("Buzhash32Reg_48".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 48))),
        ("Buzhash32Reg_64".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 64))),
        ("Buzhash32Reg_96".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 96))),
        ("Buzhash32Reg_128".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 128))),
        ("Buzhash32Reg_256".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 256))),
        ("Buzhash32Reg_512".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 512))),
        ("Buzhash32Reg_min_chunk".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, sizes.min_size()))),
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
        ("RonomonNC0".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 0))),
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 1))),
        ("RonomonNC2".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 2))),
        ("RonomonNC3".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 3))),
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
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 1))),
        ("Buzhash32Reg_64".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 64))),
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
