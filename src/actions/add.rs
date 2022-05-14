use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::staged_file;
use crate::models::tree_file;
use crate::store;

fn contains_path(m: &HashMap<&str, &tree_file::TreeFile>, p: &Path) -> bool {
    match p.to_str() {
        Some(s) => m.contains_key(s),
        None => false,
    }
}

pub fn add(
    connection: &Connection,
    full_path: &Path,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let head = db::reference::get_head(connection)?;

    let tracked_files = match head {
        Some(head_commit) => {
            println!("Current Head: {}", head_commit.hash);
            db::tree::get_tree(connection, &head_commit.hash)?
        }
        None => Vec::new(),
    };

    let mut tracked_map: HashMap<&str, &tree_file::TreeFile> = HashMap::new();
    tracked_files.iter().for_each(|tf| {
        tracked_map.insert(tf.path.as_str(), tf);
    });

    if full_path.is_dir() {
        println!("adding directory: {}", rel_path.display());
        let files: Vec<PathBuf> = staged_file::get_all(full_path).unwrap_or(vec![]);

        for file in files {
            if !contains_path(&tracked_map, &file) {
                let staged_file = staged_file::StagedFile::hash(&root_path.join(&file), &file)?;
                println!("File: {}::{}", staged_file.path, staged_file.file_hash);

                staged_file::copy_if_not_present(&staged_file, root_path)?;
                db::staging::insert(connection, &staged_file)?;
            }
        }

        return Ok(());
    }

    if full_path.is_file() {
        println!("adding file: {}", rel_path.display());
        let staged_file = staged_file::StagedFile::hash(full_path, rel_path)?;
        println!("File: {}::{}", staged_file.path, staged_file.file_hash);
        fs::copy(
            root_path.join(&staged_file.path),
            store::object_path(root_path, &staged_file.file_hash),
        )?;
        db::staging::insert(connection, &staged_file)?;
        return Ok(());
    }

    Ok(())
}
