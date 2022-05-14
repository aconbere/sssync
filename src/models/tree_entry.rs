use crate::models::file::File;
use crate::models::file_entry::FileEntry;
use std::path::PathBuf;

pub struct TreeEntry {
    pub path: String,
    pub file_hash: String,
    pub commit_hash: String,
}

impl File for TreeEntry {
    fn path_str(&self) -> &str {
        &self.path
    }
    fn path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }
    fn file_hash(&self) -> &str {
        &self.file_hash
    }
}

pub fn new(path: &str, file_hash: &str, commit_hash: &str) -> TreeEntry {
    TreeEntry {
        path: path.to_string(),
        file_hash: file_hash.to_string(),
        commit_hash: commit_hash.to_string(),
    }
}

pub fn from_file_entry(commit_hash: &str, file_entry: &FileEntry) -> TreeEntry {
    new(&file_entry.path, &file_entry.file_hash, commit_hash)
}
