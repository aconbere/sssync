use std::error::Error;
use std::fs;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::models::file_entry;
use crate::store;

pub fn add(
    connection: &Connection,
    full_path: &Path,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if full_path.is_dir() {
        println!("adding directory: {}", rel_path.display());
        let files = file_entry::get_all(full_path).unwrap_or(vec![]);
        for file in files {
            let file_entry = file_entry::FileEntry::hash(&root_path.join(&file), &file)?;
            println!("File: {}::{}", file_entry.path, file_entry.hash);

            file_entry::copy_if_not_present(&file_entry, root_path)?;
            db::staging::insert(connection, &file_entry)?;
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
