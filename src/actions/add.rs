use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::file_entry;
use crate::models::tree_entry;
use crate::store;

fn contains_path(m: &HashMap<&str, &tree_entry::TreeEntry>, p: &Path) -> bool {
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

    let mut tracked_map: HashMap<&str, &tree_entry::TreeEntry> = HashMap::new();
    tracked_files.iter().for_each(|tf| {
        tracked_map.insert(tf.path.as_str(), tf);
    });

    if full_path.is_dir() {
        println!("adding directory: {}", rel_path.display());
        let files: Vec<PathBuf> = file_entry::get_all(full_path).unwrap_or(vec![]);

        for file in files {
            if !contains_path(&tracked_map, &file) {
                let file_entry = file_entry::FileEntry::hash(&root_path.join(&file), &file)?;
                println!("File: {}::{}", file_entry.path, file_entry.hash);

                file_entry::copy_if_not_present(&file_entry, root_path)?;
                db::staging::insert(connection, &file_entry)?;
            }
        }

        return Ok(());
    }

    if full_path.is_file() {
        println!("adding file: {}", rel_path.display());
        let file_entry = file_entry::FileEntry::hash(full_path, rel_path)?;
        println!("File: {}::{}", file_entry.path, file_entry.hash);
        fs::copy(
            root_path.join(&file_entry.path),
            store::object_path(root_path, &file_entry.hash),
        )?;
        db::staging::insert(connection, &file_entry)?;
        return Ok(());
    }

    Ok(())
}
