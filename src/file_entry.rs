use crate::hash::hash_file;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileEntry {
    pub path: String,
    pub hash: String,
}

impl FileEntry {
    pub fn hash(full_path: &Path, relative_path: &Path) -> Result<Self, Box<dyn Error>> {
        println!("hash: {}", full_path.display());
        match hash_file(full_path) {
            Ok(hash) => match relative_path.to_str() {
                Some(relative_path_str) => Ok(Self {
                    path: relative_path_str.to_string(),
                    hash: hash,
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
