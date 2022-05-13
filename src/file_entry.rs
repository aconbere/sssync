use crate::hash::hash_file;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

pub struct FileEntry {
    pub path: String,
    pub hash: String,
    pub size_bytes: i64,
    pub modified_time_seconds: i64,
}

impl FileEntry {
    pub fn hash(full_path: &Path, relative_path: &Path) -> Result<Self, Box<dyn Error>> {
        println!("hash: {}", full_path.display());

        let meta = lstat(full_path)?;

        match hash_file(full_path) {
            Ok(hash) => match relative_path.to_str() {
                Some(relative_path_str) => Ok(Self {
                    path: relative_path_str.to_string(),
                    hash: hash,
                    size_bytes: meta.st_size,
                    modified_time_seconds: meta.st_mtime,
                }),
                None => Err(format!("Invalid path: {}", relative_path.display()).into()),
            },
            Err(e) => Err(e.into()),
        }
    }
}

fn default_ignore() -> HashSet<String> {
    let mut ignore = HashSet::new();
    ignore.insert(".sssync".to_string());
    ignore
}

fn should_ignore(ignore: &HashSet<String>, path: &Path) -> bool {
    match path.to_str() {
        Some(path_str) => ignore.contains(&path_str.to_string()),
        None => true,
    }
}

pub fn all_files(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    all_files_inner(root, PathBuf::from(""), &default_ignore())
}

fn all_files_inner(
    root: &Path,
    rel_path: PathBuf,
    ignore: &HashSet<String>,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    println!("all_files: {} {}", root.display(), rel_path.display());
    let mut results: Vec<PathBuf> = Vec::new();

    if should_ignore(ignore, &rel_path) {
        println!("ignoring: {}", rel_path.display());
        return Ok(results);
    }
    let contents = fs::read_dir(root)?;

    for entry in contents {
        let entry = entry?;
        let path = entry.path();
        let mut next_path = rel_path.clone();
        next_path.push(entry.file_name());

        if path.is_dir() {
            let sub_results = all_files_inner(&path, next_path, ignore)?;
            results.extend(sub_results);
        } else {
            results.push(next_path);
        }
    }
    Ok(results)
}

use errno::errno;
use std::ffi::CString;

pub fn lstat(path: &Path) -> std::io::Result<libc::stat> {
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };

    let c_path = CString::new(path.as_os_str().as_bytes())?;
    let c_errno = unsafe { libc::lstat(c_path.as_ptr(), &mut stat) };

    match c_errno {
        0 => Ok(stat),
        _ => Err(errno().into()),
    }
}

pub fn compare_file_entry(fe: &FileEntry, root_path: &Path) -> Result<bool, Box<dyn Error>> {
    let meta = lstat(Path::new(&root_path.join(&fe.path)))?;
    Ok(fe.size_bytes != meta.st_size || fe.modified_time_seconds != meta.st_mtime)
}
