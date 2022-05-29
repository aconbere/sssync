use crate::models::file::File;
use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct TreeFile {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,

    pub commit_hash: String,
}

impl PartialEq for TreeFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_hash == other.file_hash
    }
}
impl Eq for TreeFile {}

impl Hash for TreeFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.file_hash.hash(state);
    }
}

impl TreeFile {
    pub fn new(path: &str, file_hash: &str, size_bytes: i64, commit_hash: &str) -> Self {
        Self {
            path: path.to_string(),
            file_hash: file_hash.to_string(),
            size_bytes: size_bytes,
            commit_hash: commit_hash.to_string(),
        }
    }

    pub fn from_file(commit_hash: &str, file: &File) -> Self {
        Self::new(&file.path, &file.file_hash, file.size_bytes, commit_hash)
    }

    pub fn to_file(&self) -> File {
        File::new(&self.path, &self.file_hash, self.size_bytes)
    }
}
