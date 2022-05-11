use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db::{get_connection, init_db, staging_get_all, staging_insert_batch};
use crate::file_entry::{all_files, FileEntry};

pub fn add(connection: &Connection, path: &Path) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        let file_entries = all_files(path).unwrap_or(vec![]);
        staging_insert_batch(connection, file_entries)?;
        return Ok(());
    }

    if path.is_file() {
        let file_entry = FileEntry::hash(path, path)?;
        staging_insert_batch(connection, vec![file_entry])?;
        return Ok(());
    }

    Ok(())
}

pub fn status(connection: &Connection, _path: &Path) -> Result<(), Box<dyn Error>> {
    staging_get_all(connection)?;
    Ok(())
}

pub fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    let connection = get_connection(path)?;
    init_db(connection);
    Ok(())
}
