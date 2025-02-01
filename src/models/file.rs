use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::Result;

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

pub fn get_all(root: &Path) -> Result<Vec<PathBuf>> {
    get_all_inner(root, PathBuf::from(""), &default_ignore())
}

fn get_all_inner(
    root: &Path,
    rel_path: PathBuf,
    ignore: &HashSet<String>,
) -> Result<Vec<PathBuf>> {
    let mut results: Vec<PathBuf> = Vec::new();

    if should_ignore(ignore, &rel_path) {
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

pub struct FileMeta {
    pub size_bytes: i64,
    pub modified_time_seconds: i64,
}

pub fn metadata(path: &Path) -> Result<FileMeta> {
    let res = fs::symlink_metadata(path)?;

    let modified_time_seconds = res
        .modified()?
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs() as i64;

    Ok(FileMeta {
        modified_time_seconds,
        size_bytes: res.len() as i64,
    })
}
