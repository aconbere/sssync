use std::error::Error;
use std::path::{Path, PathBuf};

use crate::hash::hash_file;
use crate::models::file::lstat;
use crate::models::tree_file::TreeFile;
use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef,
};

#[derive(Debug, Clone)]
pub struct StagedFile {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,

    pub modified_time_seconds: i64,
}

impl StagedFile {
    pub fn new(
        full_path: &Path,
        relative_path: &Path,
    ) -> Result<Self, Box<dyn Error>> {
        let meta = lstat(full_path)?;

        let file_hash = hash_file(full_path)?;
        let relative_path_str = relative_path
            .to_str()
            .ok_or(format!("Invalid path: {}", relative_path.display()))?;

        Ok(Self {
            file_hash,
            path: relative_path_str.to_string(),
            size_bytes: meta.st_size,
            modified_time_seconds: meta.st_mtime,
        })
    }

    pub fn to_tree_file(&self, commit_hash: &str) -> TreeFile {
        TreeFile {
            path: self.path.clone(),
            file_hash: self.file_hash.clone(),
            size_bytes: self.size_bytes,
            commit_hash: String::from(commit_hash),
        }
    }

    // Lstat the file found at path and compare the results to the StagedFile compares size_bytes
    // and modified_time. Use this function to help avoid expensive file hashes.
    pub fn compare_lstat(&self, path: &Path) -> Result<bool, Box<dyn Error>> {
        let meta = lstat(path)?;
        Ok(self.size_bytes == meta.st_size
            && self.modified_time_seconds == meta.st_mtime)
    }
}

pub enum ChangeKind {
    Addition,
    Deletion,
}

impl ChangeKind {
    pub fn parse(s: &str) -> Result<ChangeKind, String> {
        match s {
            "addition" => Ok(ChangeKind::Addition),
            "deletion" => Ok(ChangeKind::Deletion),
            _ => Err(format!("invalid kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            ChangeKind::Addition => "addition",
            ChangeKind::Deletion => "deletion",
        }
    }
}

impl FromSql for ChangeKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match ChangeKind::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for ChangeKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

pub enum Change {
    Addition(StagedFile),
    Deletion(PathBuf),
}
