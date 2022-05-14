use crate::models::file_entry::FileEntry;

pub struct TreeEntry {
    pub path: String,
    pub file_hash: String,
    pub size_bytes: i64,

    pub commit_hash: String,
}

pub fn new(path: &str, file_hash: &str, size_bytes: i64, commit_hash: &str) -> TreeEntry {
    TreeEntry {
        path: path.to_string(),
        file_hash: file_hash.to_string(),
        size_bytes: size_bytes,
        commit_hash: commit_hash.to_string(),
    }
}

pub fn from_file_entry(commit_hash: &str, file_entry: &FileEntry) -> TreeEntry {
    new(
        &file_entry.path,
        &file_entry.file_hash,
        file_entry.size_bytes,
        commit_hash,
    )
}
