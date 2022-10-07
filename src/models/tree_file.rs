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
    pub fn update_commit_hash(&self, commit_hash: &str) -> Self {
        Self {
            path: self.path.clone(),
            file_hash: self.file_hash.clone(),
            size_bytes: self.size_bytes.clone(),
            commit_hash: String::from(commit_hash),
        }
    }
}
