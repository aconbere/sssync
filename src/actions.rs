use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::file_entry::{all_files, FileEntry};
use crate::store;

pub fn add(
    connection: &Connection,
    full_path: &Path,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if full_path.is_dir() {
        println!("adding directory: {}", rel_path.display());
        let files = all_files(full_path).unwrap_or(vec![]);
        for file in files {
            println!("f: {}", file.display());
            let file_entry = FileEntry::hash(&root_path.join(&file), &file)?;
            println!("File: {}::{}", file_entry.path, file_entry.hash);
            db::staging_insert(connection, &file_entry)?;
        }
        return Ok(());
    }

    if full_path.is_file() {
        println!("adding file: {}", rel_path.display());
        let file_entry = FileEntry::hash(full_path, rel_path)?;
        println!("File: {}::{}", file_entry.path, file_entry.hash);
        db::staging_insert(connection, &file_entry)?;
        return Ok(());
    }

    Ok(())
}

pub fn status(connection: &Connection, _path: &Path) -> Result<(), Box<dyn Error>> {
    db::staging_get_all(connection)?;
    Ok(())
}

pub fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    store::init(&path)?;

    let store_path = store::store_path(path);
    let db_path = db::db_path(&store_path);

    let connection = db::get_connection(&db_path)?;
    println!("found connection: {}", db_path.display());
    db::init(connection)?;
    Ok(())
}
