use rusqlite::params;
use rusqlite::Connection;

use std::error::Error;

use crate::models::file_entry::FileEntry;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

pub fn insert(connection: &Connection, file_entry: &FileEntry) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            staging (hash, path, size_bytes, modified_time_seconds)
        VALUES
            (?1, ?2, ?3, ?4)
        ON CONFLICT (hash)
        DO UPDATE SET
            path = excluded.path,
            size_bytes = excluded.size_bytes,
            modified_time_seconds = excluded.modified_time_seconds
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

pub fn get_all(connection: &Connection) -> Result<Vec<FileEntry>, Box<dyn Error>> {
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

pub fn delete(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
            DELETE FROM staging
        ",
        params![],
    )?;

    Ok(())
}
