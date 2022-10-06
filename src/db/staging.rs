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
                path TEXT PRIMARY KEY,
                kind TEXT,
                file_hash TEXT,
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
        Change::Addition(sf) => (
            sf.path.as_str(),
            ChangeKind::Addition,
            sf.file_hash.as_str(),
            sf.size_bytes,
            sf.modified_time_seconds,
        ),
        Change::Deletion(p) => {
            let path_str = p.to_str().unwrap();
            (path_str, ChangeKind::Deletion, "", 0, 0)
        }
    };
    connection.execute(
        "
        INSERT INTO
            staging (
                path,
                kind,
                file_hash,
                size_bytes,
                modified_time_seconds
            )
        VALUES
            (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT (path)
        DO UPDATE
        SET
            kind = excluded.kind,
            file_hash = excluded.file_hash,
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
                path,
                kind,
                file_hash,
                size_bytes,
                modified_time_seconds
            FROM
                staging
        ",
    )?;

    let entries: Vec<Change> = stmt
        .query_map([], |row| match row.get(1)? {
            ChangeKind::Addition => Ok(Change::Addition(StagedFile {
                path: row.get(0)?,
                file_hash: row.get(2)?,
                size_bytes: row.get(3)?,
                modified_time_seconds: row.get(4)?,
            })),
            ChangeKind::Deletion => {
                let p: String = row.get(0)?;
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
