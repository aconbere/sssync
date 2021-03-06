use rusqlite::params;
use rusqlite::Connection;

use std::error::Error;

use crate::models::staged_file::StagedFile;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            staging (
                file_hash TEXT PRIMARY KEY,
                path TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                modified_time_seconds INTEGER NOT NULL
            )
        ",
        params![],
    )?;

    Ok(())
}

pub fn insert(connection: &Connection, file_entry: &StagedFile) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            staging (file_hash, path, size_bytes, modified_time_seconds)
        VALUES
            (?1, ?2, ?3, ?4)
        ON CONFLICT (file_hash)
        DO UPDATE
        SET
            path = excluded.path,
            size_bytes = excluded.size_bytes,
            modified_time_seconds = excluded.modified_time_seconds
        ",
        params![
            file_entry.file_hash,
            file_entry.path,
            file_entry.size_bytes,
            file_entry.modified_time_seconds
        ],
    )?;

    Ok(())
}

pub fn get_all(connection: &Connection) -> Result<Vec<StagedFile>, Box<dyn Error>> {
    let mut stmt = connection.prepare(
        "
            SELECT
                file_hash, path, size_bytes, modified_time_seconds
            FROM
                staging
        ",
    )?;

    let entries: Vec<StagedFile> = stmt
        .query_map([], |row| {
            Ok(StagedFile {
                file_hash: row.get(0)?,
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
