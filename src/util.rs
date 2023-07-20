use std::path::{Path, PathBuf};

use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use walkdir::WalkDir;

pub const KB: usize = 1024;
pub const MB: usize = 1024 * 1024;

pub fn read_files_in_dir_sorted_by_name<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut files = read_files_in_dir_sorted(dir);
    files.sort_by_key(|(path, _)| path.clone());
    files.into_iter().map(|(path, _)| path).collect()
}

pub fn read_files_in_dir_sorted_by_size_desc<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut files = read_files_in_dir_sorted(dir);
    files.sort_by_key(|(_, size)| -(*size as i64));
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
        format!("{:.2}KB", value as f64 / KB as f64)
    } else {
        format!("{:.2}MB", value as f64 / MB as f64)
    }
}
