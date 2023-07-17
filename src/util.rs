use std::path::{Path, PathBuf};

use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use walkdir::{DirEntry, WalkDir};

pub fn read_files_in_dir_sorted<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter().filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .collect();
    files.sort();
    files
}

pub fn read_files_in_dir_split_sorted<P: AsRef<Path>>(dir: P, size_threshold: usize) -> Vec<PathBuf> {
    let (small_files, big_files): (Vec<DirEntry>, Vec<DirEntry>) = WalkDir::new(dir)
        .into_iter().filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .partition(|e| e.metadata().unwrap().len() <= size_threshold as u64);

    let mut small_files: Vec<PathBuf> = small_files.into_iter().map(|e| e.into_path()).collect();
    small_files.sort();
    let mut big_files: Vec<PathBuf> = big_files.into_iter().map(|e| e.into_path()).collect();
    big_files.sort();

    big_files.append(&mut small_files);
    big_files
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

