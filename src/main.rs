use crate::util::MB;
use benchmark::{run_benchmarks, Benchmark, NamedChunker};
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
use util::{read_files_in_dir_sorted_by_name, read_files_in_dir_sorted_by_size_desc, KB};

mod benchmark;
mod chunkers;
mod util;

fn main() -> std::io::Result<()> {
    let standard_chunkers: Vec<NamedChunker> = vec![
        ("FixedSize".to_string(), |_| Box::new(Fixed::new())),
        ("FastCdc2016".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 2))),
        ("FastCdc2020".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 2))),
        ("Restic".to_string(), |sizes| Box::new(ResticCdc::new(Pol::generate_random(), sizes))),
        ("StadiaCdc".to_string(), |sizes| Box::new(GoogleStadiaCdc::new(sizes))),
        ("Casync".to_string(), |sizes| Box::new(Casync::new(sizes))),
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 1))),
    ];
    let fast_cdc_chunkers: Vec<NamedChunker> = vec![
        ("FastCdc2016NC1".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 1))),
        ("FastCdc2016".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 2))),
        ("FastCdc2016NC3".to_string(), |sizes| Box::new(FastCdc2016::new(sizes, 3))),
        ("FastCdc2020NC1".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 1))),
        ("FastCdc2020".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 2))),
        ("FastCdc2020NC3".to_string(), |sizes| Box::new(FastCdc2020::new(sizes, 3))),
        ("Ronomon".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 1))),
        ("RonomonNC2".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 2))),
        ("RonomonNC3".to_string(), |sizes| Box::new(RonomonCdc::new(sizes, 3))),
        ("Ronomon64".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 1))),
        ("Ronomon64NC2".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 2))),
        ("Ronomon64NC3".to_string(), |sizes| Box::new(Ronomon64Cdc::new(sizes, 3))),
    ];
    let buzhash_chunkers: Vec<NamedChunker> = vec![
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
        ("Buzhash64Reg_64".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 64))),
        ("Buzhash64Reg_96".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 96))),
        ("Buzhash64Reg_128".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 128))),
        ("Buzhash64Reg_256".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 256))),
        ("Buzhash64Reg_512".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, 512))),
        ("Buzhash64Reg_min_chunk".to_string(), |sizes| Box::new(Buzhash64Reg::new(sizes, sizes.min_size()))),
    ];

    let benchmarks = vec![
        Benchmark::new(
            "64KB_FastCdc".to_string(),
            64 * KB,
            fast_cdc_chunkers.clone(),
            read_files_in_dir_sorted_by_name,
        ),
        Benchmark::new("64KB_Buz".to_string(), 64 * KB, buzhash_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "64KB_fsd".to_string(),
            64 * KB,
            standard_chunkers.clone(),
            read_files_in_dir_sorted_by_size_desc,
        ),
        Benchmark::new("64KB".to_string(), 64 * KB, standard_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "128KB_FastCdc".to_string(),
            128 * KB,
            fast_cdc_chunkers.clone(),
            read_files_in_dir_sorted_by_name,
        ),
        Benchmark::new("128KB_Buz".to_string(), 128 * KB, buzhash_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "128KB_fsd".to_string(),
            128 * KB,
            standard_chunkers.clone(),
            read_files_in_dir_sorted_by_size_desc,
        ),
        Benchmark::new("128KB".to_string(), 128 * KB, standard_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "256KB_FastCdc".to_string(),
            256 * KB,
            fast_cdc_chunkers.clone(),
            read_files_in_dir_sorted_by_name,
        ),
        Benchmark::new("256KB_Buz".to_string(), 256 * KB, buzhash_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "256KB_fsd".to_string(),
            256 * KB,
            standard_chunkers.clone(),
            read_files_in_dir_sorted_by_size_desc,
        ),
        Benchmark::new("256KB".to_string(), 256 * KB, standard_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "512KB_FastCdc".to_string(),
            512 * KB,
            fast_cdc_chunkers.clone(),
            read_files_in_dir_sorted_by_name,
        ),
        Benchmark::new("512KB_Buz".to_string(), 512 * KB, buzhash_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new(
            "512KB_fsd".to_string(),
            512 * KB,
            standard_chunkers.clone(),
            read_files_in_dir_sorted_by_size_desc,
        ),
        Benchmark::new("512KB".to_string(), 512 * KB, standard_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new("1MB_FastCdc".to_string(), 1 * MB, fast_cdc_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new("1MB_Buz".to_string(), 1 * MB, buzhash_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new("1MB_fsd".to_string(), 1 * MB, standard_chunkers.clone(), read_files_in_dir_sorted_by_size_desc),
        Benchmark::new("1KB".to_string(), 1 * MB, standard_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new("2MB_FastCdc".to_string(), 2 * MB, fast_cdc_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new("2MB_Buz".to_string(), 2 * MB, buzhash_chunkers.clone(), read_files_in_dir_sorted_by_name),
        Benchmark::new("2MB_fsd".to_string(), 2 * MB, standard_chunkers.clone(), read_files_in_dir_sorted_by_size_desc),
        Benchmark::new("2MB".to_string(), 2 * MB, standard_chunkers.clone(), read_files_in_dir_sorted_by_name),
    ];
    run_benchmarks(benchmarks)?;

    Ok(())
}
