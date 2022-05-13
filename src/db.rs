use rusqlite::params;
use rusqlite::Connection;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::file_entry::FileEntry;
use crate::store;

pub const DB_FILE_NAME: &str = "sssync.db";

pub fn db_path(path: &Path) -> PathBuf {
    path.join(DB_FILE_NAME)
}

pub fn has_db_file(path: &Path) -> bool {
    path.join(DB_FILE_NAME).exists()
}

pub fn get_connection(root_path: &Path) -> Result<Connection, Box<dyn Error>> {
    println!("get_connection: {}", root_path.display());
    let store_path = store::store_path(root_path);
    let db_path = db_path(&store_path);
    println!("opening db: {}", db_path.display());
    let connection = Connection::open(db_path)?;
    Ok(connection)
}

pub fn init(connection: Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            objects (
                hash TEXT PRIMARY KEY, path TEXT NOT NULL
            )
        ",
        params![],
    )?;

    connection.execute(
        "
        CREATE TABLE
            staging (
                hash TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                modified_time_seconds INTEGER NOT NULL
            )
        ",
        params![],
    )?;

    connection.execute(
        "
        CREATE TABLE
            commits (
                hash TEXT PRIMARY KEY
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn staging_get_all(connection: &Connection) -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let mut stmt = connection.prepare(
        "
            SELECT
                hash, path, size_bytes, modified_time_seconds
            FROM
                staging
        ",
    )?;
    let entries: Vec<FileEntry> = stmt
        .query_map([], |row| {
            Ok(FileEntry {
                hash: row.get(0)?,
                path: row.get(1)?,
                size_bytes: row.get(2)?,
                modified_time_seconds: row.get(3)?,
            })
        })?
        .filter_map(|fe| fe.ok())
        .collect();

    Ok(entries)
}

pub fn staging_insert(
    connection: &Connection,
    file_entry: &FileEntry,
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            staging (hash, path, size_bytes, modified_time_seconds)
        VALUES
            (?1, ?2, ?3, ?4)
        ON CONFLICT (hash)
        DO UPDATE SET
            path = excluded.path
        ",
        params![
            file_entry.hash,
            file_entry.path,
            file_entry.size_bytes,
            file_entry.modified_time_seconds
        ],
    )?;
    Ok(())
}

pub fn staging_insert_batch(
    connection: &Connection,
    file_entries: Vec<FileEntry>,
) -> Result<(), Box<dyn Error>> {
    for file_entry in file_entries {
        staging_insert(connection, &file_entry)?;
    }
    Ok(())
}
