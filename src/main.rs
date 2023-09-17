use std::path::{Path, PathBuf};
use std::u128;

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
use crate::chunkers::{new_adler_u32, new_buz, new_buz_spread_mask, new_gear_spread_mask, new_polynomial};
use crate::hashes::polynomial_hash::polynomial::Pol;
use crate::hashes::tables::{buz_table, sha256_u128_table, sha256_u32_table, sha256_u64_table};
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
        ("FastCdc2016 nc3".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 3))),
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
        ("Buzhash32 48".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 48, 0))),
        ("Buzhash32 48 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u32_table(), 48, 1))),
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
        // Buzhash 32 balanced table
        ("Buzhash32b 63".to_string(), |sizes| Box::new(new_buz::<u32>(sizes, buz_table(), 63, 0))),
        ("Buzhash32b 63 nc1".to_string(), |sizes| Box::new(new_buz::<u32>(sizes, buz_table(), 63, 1))),
        ("Buzhash32b 63 spread".to_string(), |sizes| Box::new(new_buz_spread_mask::<u32>(sizes, buz_table(), 63, 0))),
        ("Buzhash32b 64".to_string(), |sizes| Box::new(new_buz::<u32>(sizes, buz_table(), 64, 0))),
        ("Buzhash32b 64 nc1".to_string(), |sizes| Box::new(new_buz::<u32>(sizes, buz_table(), 64, 1))),
        ("Buzhash32b 64 spread".to_string(), |sizes| Box::new(new_buz_spread_mask::<u32>(sizes, buz_table(), 64, 0))),
        // Buzhash 64
        ("Buzhash64 31".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 31, 0))),
        ("Buzhash64 31 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 31, 1))),
        ("Buzhash64 32".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 32, 0))),
        ("Buzhash64 32 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 32, 1))),
        ("Buzhash64 48".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 48, 0))),
        ("Buzhash64 48 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u64_table(), 48, 1))),
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
        // Buzhash 64 balanced table
        ("Buzhash64b 127".to_string(), |sizes| Box::new(new_buz::<u64>(sizes, buz_table(), 127, 0))),
        ("Buzhash64b 127 nc1".to_string(), |sizes| Box::new(new_buz::<u64>(sizes, buz_table(), 127, 1))),
        ("Buzhash64b 127 spread".to_string(), |sizes| Box::new(new_buz_spread_mask::<u64>(sizes, buz_table(), 127, 0))),
        ("Buzhash64b 128".to_string(), |sizes| Box::new(new_buz::<u64>(sizes, buz_table(), 128, 0))),
        ("Buzhash64b 128 nc1".to_string(), |sizes| Box::new(new_buz::<u64>(sizes, buz_table(), 128, 1))),
        ("Buzhash64b 128 spread".to_string(), |sizes| Box::new(new_buz_spread_mask::<u64>(sizes, buz_table(), 128, 0))),
        // Buzhash 128
        ("Buzhash128 31".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 31, 0))),
        ("Buzhash128 31 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 31, 1))),
        ("Buzhash128 32".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 32, 0))),
        ("Buzhash128 32 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 32, 1))),
        ("Buzhash128 48".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 48, 0))),
        ("Buzhash128 48 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 48, 1))),
        ("Buzhash128 63".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 63, 0))),
        ("Buzhash128 63 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 63, 1))),
        ("Buzhash128 64".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 64, 0))),
        ("Buzhash128 64 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 64, 1))),
        ("Buzhash128 128".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 128, 0))),
        ("Buzhash128 128 nc1".to_string(), |sizes| Box::new(new_buz(sizes, sha256_u128_table(), 128, 1))),
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
        // Buzhash 128 balanced table
        ("Buzhash128b 128".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 128, 0))),
        ("Buzhash128b 128 nc1".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 128, 1))),
        ("Buzhash128b 255".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 255, 0))),
        ("Buzhash128b 255 nc1".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 255, 1))),
        ("Buzhash128b 255 spread".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 255, 0))
        }),
        ("Buzhash128b 256".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 256, 0))),
        ("Buzhash128b 256 nc1".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 256, 1))),
        ("Buzhash128b 256 nc2".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 256, 2))),
        ("Buzhash128b 256 spread".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 256, 0))
        }),
        ("Buzhash128b 256 spread nc1".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 256, 1))
        }),
        ("Buzhash128b 256 spread nc2".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 256, 2))
        }),
        ("Buzhash128b 512".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 512, 0))),
        ("Buzhash128b 512 nc1".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 512, 1))),
        ("Buzhash128b 512 nc2".to_string(), |sizes| Box::new(new_buz::<u128>(sizes, buz_table(), 512, 2))),
        ("Buzhash128b 512 spread".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 512, 0))
        }),
        ("Buzhash128b 512 spread nc1".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 512, 1))
        }),
        ("Buzhash128b 512 spread nc2".to_string(), |sizes| {
            Box::new(new_buz_spread_mask::<u128>(sizes, buz_table(), 512, 2))
        }),
        // Gear
        ("Gear64 nc2 Buz table".to_string(), |sizes| Box::new(new_gear_spread_mask::<u64>(sizes, buz_table(), 2))),
        ("Gear128 nc0 Buz table".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, buz_table(), 1))),
        ("Gear128 nc1 Buz table".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, buz_table(), 2))),
        ("Gear128 nc2 Buz table".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, buz_table(), 3))),
        ("Gear128 nc3 Buz table".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, buz_table(), 4))),
        ("Gear128 nc0".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, sha256_u128_table(), 1))),
        ("Gear128 nc1".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, sha256_u128_table(), 2))),
        ("Gear128 nc2".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, sha256_u128_table(), 3))),
        ("Gear128 nc3".to_string(), |sizes| Box::new(new_gear_spread_mask::<u128>(sizes, sha256_u128_table(), 4))),
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
        // Polynomial
        ("Polynomial 31".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 31, 0))),
        ("Polynomial 31 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 31, 1))),
        ("Polynomial 32".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 32, 0))),
        ("Polynomial 32 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 32, 1))),
        ("Polynomial 48".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 48, 0))),
        ("Polynomial 48 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 48, 1))),
        ("Polynomial 63".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 63, 0))),
        ("Polynomial 63 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 63, 1))),
        ("Polynomial 64".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 64, 0))),
        ("Polynomial 64 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 64, 1))),
        ("Polynomial 255".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 255, 0))),
        ("Polynomial 255 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 255, 1))),
        ("Polynomial 255 nc2".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 255, 2))),
        ("Polynomial 256".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 256, 0))),
        ("Polynomial 256 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 256, 1))),
        ("Polynomial 256 nc2".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 256, 2))),
        ("Polynomial 511".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 511, 0))),
        ("Polynomial 511 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 511, 1))),
        ("Polynomial 511 nc2".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 511, 2))),
        ("Polynomial 512".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 512, 0))),
        ("Polynomial 512 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 512, 1))),
        ("Polynomial 512 nc2".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 512, 2))),
        ("Polynomial 4095".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 4095, 0))),
        ("Polynomial 4095 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 4095, 1))),
        ("Polynomial 4095 nc2".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 4095, 2))),
        ("Polynomial 4096".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 4096, 0))),
        ("Polynomial 4096 nc1".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 4096, 1))),
        ("Polynomial 4096 nc2".to_string(), |sizes| Box::new(new_polynomial(sizes, Pol::generate_random(), 4096, 2))),
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
