use std::error::Error;
use std::path::Path;

use crate::hash::hash_file;
use crate::models::file::{lstat, File};

pub struct StagedFile {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,

    pub modified_time_seconds: i64,
}

impl StagedFile {
    pub fn new(full_path: &Path, relative_path: &Path) -> Result<Self, Box<dyn Error>> {
        let meta = lstat(full_path)?;

        match hash_file(full_path) {
            Ok(file_hash) => match relative_path.to_str() {
                Some(relative_path_str) => Ok(Self {
                    path: relative_path_str.to_string(),
                    file_hash: file_hash,
                    size_bytes: meta.st_size,
                    modified_time_seconds: meta.st_mtime,
                }),
                None => Err(format!("Invalid path: {}", relative_path.display()).into()),
            },
            Err(e) => Err(e.into()),
        }
    }

    pub fn to_file(&self) -> File {
        File {
            path: self.path.clone(),
            file_hash: self.file_hash.clone(),
            size_bytes: self.size_bytes,
        }
    }
}

pub fn compare_file_meta(fe: &StagedFile, root_path: &Path) -> Result<bool, Box<dyn Error>> {
    let meta = lstat(Path::new(&root_path.join(&fe.path)))?;
    Ok(fe.size_bytes != meta.st_size || fe.modified_time_seconds != meta.st_mtime)
}
