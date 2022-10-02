use std::error::Error;
use std::path::Path;

use crate::hash::hash_file;
use crate::models::file::{lstat, File};

#[derive(Debug, Clone)]
pub struct StagedFile {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,
    pub modified_time_seconds: i64,
}

impl StagedFile {
    pub fn new(full_path: &Path, relative_path: &Path) -> Result<Self, Box<dyn Error>> {
        let meta = lstat(full_path)?;

        let file_hash = hash_file(full_path)?;
        let relative_path_str = relative_path
            .to_str()
            .ok_or(format!("Invalid path: {}", relative_path.display()))?;

        Ok(Self {
            path: relative_path_str.to_string(),
            file_hash: file_hash,
            size_bytes: meta.st_size,
            modified_time_seconds: meta.st_mtime,
        })
    }

    pub fn to_file(&self) -> File {
        File {
            path: self.path.clone(),
            file_hash: self.file_hash.clone(),
            size_bytes: self.size_bytes,
        }
    }

    // Lstat the file found at path and compare the results to the StagedFile compares size_bytes
    // and modified_time. Use this function to help avoid expensive file hashes.
    pub fn compare_lstat(&self, path: &Path) -> Result<bool, Box<dyn Error>> {
        let meta = lstat(path)?;
        Ok(self.size_bytes == meta.st_size && self.modified_time_seconds == meta.st_mtime)
    }
}
