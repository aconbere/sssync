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
        "CREATE TABLE objects (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;

    connection.execute(
        "CREATE TABLE staging (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;

    connection.execute("CREATE TABLE commits (hash TEXT primary key)", params![])?;
    Ok(())
}

pub fn staging_get_all(connection: &Connection) -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let mut stmt = connection.prepare("SELECT hash, path FROM staging")?;
    let entries: Vec<FileEntry> = stmt
        .query_map([], |row| {
            Ok(FileEntry {
                hash: row.get(0)?,
                path: row.get(1)?,
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
        "INSERT INTO staging (hash, path) VALUES (?1, ?2) ON CONFLICT (hash) DO UPDATE SET path = excluded.path",
        params![file_entry.hash, file_entry.path],
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
