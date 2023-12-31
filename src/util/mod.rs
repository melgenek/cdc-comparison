use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use walkdir::WalkDir;

pub mod chunk_sizes;
pub mod chunk_stream;
pub mod mask_builder;
pub mod multi_file_dir;
pub mod unsigned_integer;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;

pub fn read_files_in_dir_sorted_by_name<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut files = read_files_in_dir_sorted(dir);
    files.sort_by_key(|(path, _)| path.clone());
    files.into_iter().map(|(path, _)| path).collect()
}

fn read_files_in_dir_sorted<P: AsRef<Path>>(dir: P) -> Vec<(PathBuf, u64)> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| {
            let len = e.metadata().unwrap().len();
            (e.into_path(), len)
        })
        .collect()
}

pub fn sha256(bytes: &[u8]) -> String {
    let mut context = Context::new(&SHA256);
    context.update(&bytes);
    HEXLOWER.encode(context.finish().as_ref())
}

pub fn sha256_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut context = Context::new(&SHA256);
    let mut buffer = vec![0; 1 * MB];
    let mut file = File::open(path)?;
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok(HEXLOWER.encode(context.finish().as_ref()))
}

/// Base-2 logarithm
pub fn logarithm2(value: u32) -> u32 {
    f64::from(value).log2().round() as u32
}

pub fn size_to_str(value: usize) -> String {
    if value < MB {
        format!("{}KB", value as f64 / KB as f64)
    } else {
        format!("{}MB", value as f64 / MB as f64)
    }
}

pub fn size_to_str_f64(value: f64) -> String {
    if value < MB as f64 {
        format!("{:.2}KB", value / KB as f64)
    } else {
        format!("{:.2}MB", value / MB as f64)
    }
}
