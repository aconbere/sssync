use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};

use crate::models::status::Hashable;

#[derive(Clone, Debug, Hash)]
pub struct TreeFile {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,

    pub commit_hash: String,
}

impl PartialEq for TreeFile {
    fn eq(&self, other: &Self) -> bool {
        self.file_hash == other.file_hash && self.path == other.path
    }
}

impl Eq for TreeFile {}

impl TreeFile {
    pub fn update_commit_hash(&self, commit_hash: &str) -> Self {
        Self {
            path: self.path.clone(),
            file_hash: self.file_hash.clone(),
            size_bytes: self.size_bytes,
            commit_hash: String::from(commit_hash),
        }
    }
}

impl Hashable for TreeFile {
    fn file_hash(&self) -> String {
        self.file_hash.clone()
    }
}

pub struct TreeFileFileHash(pub TreeFile);

impl PartialEq for TreeFileFileHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.file_hash == other.0.file_hash
    }
}

impl Eq for TreeFileFileHash {}

impl Hash for TreeFileFileHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.file_hash.hash(state);
    }
}

pub struct TreeFilePathHash(pub TreeFile);

impl PartialEq for TreeFilePathHash {
    fn eq(&self, other: &Self) -> bool {
        self.0.path == other.0.path
    }
}

impl Eq for TreeFilePathHash {}

impl Hash for TreeFilePathHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.path.hash(state);
    }
}
