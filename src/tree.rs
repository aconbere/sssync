pub struct TreeEntry {
    pub path: String,
    pub file_hash: String,
    pub commit_hash: String,
}

pub fn new(path, file_hash, commit_hash) -> TreeEntry {
    TreeEntry {
        path: path,
        file_hash: file_hash,
        commit_hash: commit_hash,
    }
}
