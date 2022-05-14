use std::collections::HashSet;
use std::error::Error;
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use errno::errno;

use crate::hash::hash_string;
use crate::store;

pub struct File {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,
}

pub fn hash_all(files: &Vec<File>) -> String {
    hash_string(files.iter().map(|f| f.file_hash.as_str()).collect())
}

pub fn copy_if_not_present(file: &File, root_path: &Path) -> Result<(), Box<dyn Error>> {
    let full_path = root_path.join(&file.path);

    if !full_path.exists() {
        fs::copy(full_path, store::object_path(root_path, &file.file_hash))?;
    }
    Ok(())
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

pub fn get_all(root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    get_all_inner(root, PathBuf::from(""), &default_ignore())
}

fn get_all_inner(
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
            let sub_results = get_all_inner(&path, next_path, ignore)?;
            results.extend(sub_results);
        } else {
            results.push(next_path);
        }
    }
    Ok(results)
}

pub fn lstat(path: &Path) -> std::io::Result<libc::stat> {
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };

    let c_path = CString::new(path.as_os_str().as_bytes())?;
    let c_errno = unsafe { libc::lstat(c_path.as_ptr(), &mut stat) };

    match c_errno {
        0 => Ok(stat),
        _ => Err(errno().into()),
    }
}
