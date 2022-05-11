use crate::hash::hash_file;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileEntry {
    pub path: String,
    pub hash: String,
}

impl FileEntry {
    pub fn hash(full_path: &Path, relative_path: &Path) -> Result<Self, Box<dyn Error>> {
        match hash_file(full_path) {
            Ok(hash) => match relative_path.to_str() {
                Some(relative_path_str) => Ok(Self {
                    path: relative_path_str.to_string(),
                    hash: hash,
                }),
                None => Err(format!("Invalid path: {}", relative_path.display()).into()),
            },
            Err(e) => Err(e.into()),
        }
    }
}

pub fn all_files(root: &Path) -> Result<Vec<FileEntry>, Box<dyn Error>> {
    all_files_inner(root, PathBuf::from("./"))
}

fn all_files_inner(root: &Path, up_to_path: PathBuf) -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let contents = fs::read_dir(root)?;
    let mut results: Vec<FileEntry> = Vec::new();

    for entry in contents {
        let entry = entry?;
        let path = entry.path();
        let mut relative_path = up_to_path.clone();
        relative_path.push(entry.file_name());

        if path.is_dir() {
            let sub_results = all_files_inner(&path, relative_path)?;
            results.extend(sub_results);
        } else {
            let file_entry = FileEntry::hash(path.as_path(), relative_path.as_path())?;
            results.push(file_entry);
        }
    }
    Ok(results)
}
