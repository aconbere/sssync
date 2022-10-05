use rusqlite::params;
use rusqlite::Connection;
use std::error::Error;
use std::path::PathBuf;

use crate::models::staged_file::{Change, ChangeKind, StagedFile};

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            staging (
                kind TEXT,
                path TEXT NOT NULL,
                file_hash TEXT PRIMARY KEY,
                size_bytes INTEGER NOT NULL,
                modified_time_seconds INTEGER NOT NULL
            )
        ",
        params![],
    )?;

    Ok(())
}

pub fn insert(
    connection: &Connection,
    change: &Change,
) -> Result<(), Box<dyn Error>> {
    let params = match change {
        Change::Addition(sf) => {
            params![
                ChangeKind::Addition,
                sf.file_hash,
                sf.path,
                sf.size_bytes,
                sf.modified_time_seconds
            ]
        }
        Change::Deletion(p) => {
            let path_str = String::from(p.to_str().unwrap());
            params![ChangeKind::Deletion, "", path_str, 0, 0,]
        }
    };
    connection.execute(
        "
        INSERT INTO
            staging (
                kind,
                path,
                file_hash,
                size_bytes,
                modified_time_seconds
            )
        VALUES
            (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT (kind, file_hash)
        DO UPDATE
        SET
            path = excluded.path,
            size_bytes = excluded.size_bytes,
            modified_time_seconds = excluded.modified_time_seconds
        ",
        params,
    )?;

    Ok(())
}

pub fn get_all(connection: &Connection) -> Result<Vec<Change>, Box<dyn Error>> {
    let mut stmt = connection.prepare(
        "
            SELECT
                kind,
                file_hash,
                path,
                size_bytes,
                modified_time_seconds
            FROM
                staging
        ",
    )?;

    let entries: Vec<Change> = stmt
        .query_map([], |row| match row.get(0)? {
            ChangeKind::Addition => Ok(Change::Addition(StagedFile {
                file_hash: row.get(1)?,
                path: row.get(2)?,
                size_bytes: row.get(3)?,
                modified_time_seconds: row.get(4)?,
            })),
            ChangeKind::Deletion => {
                let p: String = row.get(2)?;
                Ok(Change::Deletion(PathBuf::from(p)))
            }
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
