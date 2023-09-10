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
use util::{read_files_in_dir_sorted_by_name, KB};

use crate::benchmark::{avg_to_standard_sizes, evaluate, evaluate_full_files};
use crate::chunkers::ported::borg::Borg;
use crate::chunkers::ported::pci::Pci;
use crate::chunkers::ported::restic::ResticCdc;
use crate::chunkers::{new_adler_u32, new_buz, new_buz_spread_mask, new_gear_u128};
use crate::hashes::polynomial_hash::polynomial::Pol;
use crate::hashes::tables::{sha256_u128_table, sha256_u32_table, sha256_u64_table};
use crate::util::MB;

mod benchmark;
mod chunkers;
mod hashes;
mod util;

fn main() -> std::io::Result<()> {
    evaluate_full_files(
        vec![
            PathBuf::from("data/extracted/postgres-15.2-extracted"),
            PathBuf::from("data/extracted/postgres-15.3-extracted"),
        ],
        Path::new("results"),
    )?;
    evaluate_chunkers()?;
    Ok(())
}

fn evaluate_chunkers() -> std::io::Result<()> {
    let chunkers: Vec<NamedChunker> = vec![
        ("FixedSize".to_string(), |_| Box::new(Fixed::new())),
        ("Borg".to_string(), |sizes| Box::new(Borg::new(sizes))),
        ("Casync".to_string(), |sizes| Box::new(Casync::new(sizes))),
        // fastcdc
        ("FastCdc2016".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 2))),
        ("FastCdc2016 nc1".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 1))),
        ("FastCdc2016 nc0".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 0))),
        ("FastCdc2020".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 2))),
        // Stadia
        ("StadiaCdc".to_string(), |sizes| Box::new(GoogleStadiaCdc::new(sizes))),
        // pci
        ("Pci 5".to_string(), |sizes| Box::new(Pci::new(sizes, 5, 0))),
        ("Pci 512".to_string(), |sizes| Box::new(Pci::new(sizes, 512, 0))),
        ("Pci 4096".to_string(), |sizes| Box::new(Pci::new(sizes, 4096, 0))),
        ("Pci 4096 nc1".to_string(), |sizes| Box::new(Pci::new(sizes, 4096, 1))),
        ("Pci min".to_string(), |sizes| Box::new(Pci::new(sizes, sizes.min_size(), 0))),
        // Restic
        ("Restic".to_string(), |sizes| Box::new(ResticCdc::new(Pol::generate_random(), sizes))),
        // ronomon
        ("Ronomon nc0".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 0))),
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 1))),
        ("Ronomon nc2".to_string(), |sizes| Box::new(RonomonCdc::new_original(sizes, 2))),
        ("Ronomon64 nc0".to_string(), |sizes| Box::new(RonomonCdc::new_u64(sizes, 0))),
        ("Ronomon64 nc2".to_string(), |sizes| Box::new(RonomonCdc::new_u64(sizes, 2))),
        ("Ronomon64 nc1".to_string(), |sizes| Box::new(RonomonCdc::new_u64(sizes, 1))),
        // Buzhash reg
        ("Buzhash32Reg 48".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 48))),
        ("Buzhash32Reg 64".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 64))),
        ("Buzhash32Reg 256".to_string(), |sizes| Box::new(Buzhash32Reg::new(sizes, 64))),
        ("Buzhash64Reg 48".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 48))),
        ("Buzhash64Reg 64".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 64))),
        ("Buzhash64Reg 256".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 256))),
        // Buzhash 32
        ("Buzhash32 31".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 31, 0))),
        ("Buzhash32 31 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 31, 1))),
        ("Buzhash32 32".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 32, 0))),
        ("Buzhash32 32 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 32, 1))),
        ("Buzhash32 63".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 63, 0))),
        ("Buzhash32 63 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 63, 1))),
        ("Buzhash32 64".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 64, 0))),
        ("Buzhash32 64 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 64, 1))),
        ("Buzhash32 255".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 255, 0))),
        ("Buzhash32 255 spread".to_string(), |sizes| Box::new(new_buz_spread_mask(sizes, sha256_u32_table(), 255, 0))),
        ("Buzhash32 255 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 255, 1))),
        ("Buzhash32 255 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 255, 2))),
        ("Buzhash32 256".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 256, 0))),
        ("Buzhash32 256 spread".to_string(), |sizes| Box::new(new_buz_spread_mask(sizes, sha256_u32_table(), 256, 0))),
        ("Buzhash32 256 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 256, 1))),
        ("Buzhash32 256 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 256, 2))),
        ("Buzhash32 4095".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 4095, 0))),
        ("Buzhash32 4095 spread".to_string(), |sizes| {
            Box::new(new_buz_spread_mask(sizes, sha256_u32_table(), 4095, 0))
        }),
        ("Buzhash32 4095 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 4095, 1))),
        ("Buzhash32 4095 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 4095, 2))),
        ("Buzhash32 4096".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 4096, 0))),
        ("Buzhash32 4096 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 4096, 1))),
        ("Buzhash32 4096 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 4096, 2))),
        // Buzhash 64
        ("Buzhash64 31".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 31, 0))),
        ("Buzhash64 31 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 31, 1))),
        ("Buzhash64 32".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 32, 0))),
        ("Buzhash64 32 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 32, 1))),
        ("Buzhash64 63".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 63, 0))),
        ("Buzhash64 63 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 63, 1))),
        ("Buzhash64 64".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 64, 0))),
        ("Buzhash64 64 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 64, 1))),
        ("Buzhash64 255".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 255, 0))),
        ("Buzhash64 255 spread".to_string(), |sizes| Box::new(new_buz_spread_mask(sizes, sha256_u64_table(), 255, 0))),
        ("Buzhash64 255 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 255, 1))),
        ("Buzhash64 255 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 255, 2))),
        ("Buzhash64 256".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 256, 0))),
        ("Buzhash64 256 spread".to_string(), |sizes| Box::new(new_buz_spread_mask(sizes, sha256_u64_table(), 256, 0))),
        ("Buzhash64 256 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 256, 1))),
        ("Buzhash64 256 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 256, 2))),
        ("Buzhash64 4095".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 4095, 0))),
        ("Buzhash64 4095 spread".to_string(), |sizes| {
            Box::new(new_buz_spread_mask(sizes, sha256_u64_table(), 4095, 0))
        }),
        ("Buzhash64 4095 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 4095, 1))),
        ("Buzhash64 4095 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 4095, 2))),
        ("Buzhash64 4096".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 4096, 0))),
        ("Buzhash64 4096 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 4096, 1))),
        ("Buzhash64 4096 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 4096, 2))),
        // Buzhash 128
        ("Buzhash128 31".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 31, 0))),
        ("Buzhash128 31 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 31, 1))),
        ("Buzhash128 32".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 32, 0))),
        ("Buzhash128 32 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 32, 1))),
        ("Buzhash128 63".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 63, 0))),
        ("Buzhash128 63 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 63, 1))),
        ("Buzhash128 64".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 64, 0))),
        ("Buzhash128 64 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 64, 1))),
        ("Buzhash128 255".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 255, 0))),
        ("Buzhash128 255 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 255, 1))),
        ("Buzhash128 255 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 255, 2))),
        ("Buzhash128 256".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 256, 0))),
        ("Buzhash128 256 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 256, 1))),
        ("Buzhash128 256 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 256, 2))),
        ("Buzhash128 511".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 511, 0))),
        ("Buzhash128 511 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 511, 1))),
        ("Buzhash128 511 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 511, 2))),
        ("Buzhash128 512".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 512, 0))),
        ("Buzhash128 512 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 512, 1))),
        ("Buzhash128 512 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 512, 2))),
        ("Buzhash128 4095".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 4095, 0))),
        ("Buzhash128 4095 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 4095, 1))),
        ("Buzhash128 4095 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 4095, 2))),
        ("Buzhash128 4096".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 4096, 0))),
        ("Buzhash128 4096 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 4096, 1))),
        ("Buzhash128 4096 nc2".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 4096, 2))),
        // Gear
        ("Gear128 nc0".to_string(), |sizes| Box::new(new_gear_u128(sizes, 0))),
        ("Gear128 nc1".to_string(), |sizes| Box::new(new_gear_u128(sizes, 1))),
        ("Gear128 nc2".to_string(), |sizes| Box::new(new_gear_u128(sizes, 2))),
        ("Gear128 nc3".to_string(), |sizes| Box::new(new_gear_u128(sizes, 3))),
        // Adler32
        ("Adler32 32 nc0".to_string(), |sizes| Box::new(new_adler_u32(sizes, 32, 0))),
        ("Adler32 32 nc1".to_string(), |sizes| Box::new(new_adler_u32(sizes, 32, 1))),
        ("Adler32 64 nc0".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 0))),
        ("Adler32 64 nc1".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 1))),
        ("Adler32 256 nc0".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 0))),
        ("Adler32 256 nc1".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 1))),
        ("Adler32 1024 nc0".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 0))),
        ("Adler32 1024 nc1".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 1))),
        ("Adler32 4096 nc0".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 0))),
        ("Adler32 4096 nc1".to_string(), |sizes| Box::new(new_adler_u32(sizes, 64, 1))),
    ];

    let input_dirs: Vec<PathBuf> = vec![
        PathBuf::from("data/concatenated/postgres-15.2.tar"),
        PathBuf::from("data/concatenated/postgres-15.3.tar"),
    ];
    let avg_sizes = vec![64 * KB, 128 * KB, 256 * KB, 512 * KB, 1 * MB, 2 * MB];
    evaluate(
        avg_sizes.clone(),
        avg_to_standard_sizes,
        chunkers.clone(),
        read_files_in_dir_sorted_by_name,
        input_dirs.clone(),
        Path::new("results/json"),
    )?;
    Ok(())
}
