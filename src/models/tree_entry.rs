pub struct TreeEntry {
    pub path: String,
    pub file_hash: String,
    pub commit_hash: String,
}

pub fn new(path: &str, file_hash: &str, commit_hash: &str) -> TreeEntry {
    TreeEntry {
        path: path.to_string(),
        file_hash: file_hash.to_string(),
        commit_hash: commit_hash.to_string(),
    }
}
