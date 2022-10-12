use std::error::Error;

use rusqlite;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::reference::{Kind, Reference, LOCAL};

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            refs (
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                hash TEXT NOT NULL,
                remote TEXT,
                PRIMARY KEY (name, kind, remote)
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(
    connection: &Connection,
    name: &str,
    kind: Kind,
    hash: &str,
    remote: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT OR IGNORE INTO
            refs (name, kind, hash, remote)
        VALUES
            (?1, ?2, ?3)
        ",
        params![name, kind, hash, remote.unwrap_or(LOCAL)],
    )?;
    Ok(())
}

pub fn update(
    connection: &Connection,
    name: &str,
    kind: Kind,
    hash: &str,
    remote: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            refs (name, kind, hash, remote)
        VALUES
            (?1, ?2, ?3, ?4)
        ON CONFLICT (name, kind, remote)
        DO UPDATE
        SET
            hash = excluded.hash
        ",
        params![name, kind, hash, remote.unwrap_or(LOCAL)],
    )?;
    Ok(())
}

pub fn get_all_by_kind(
    connection: &Connection,
    remote: Option<&str>,
    kind: Kind,
) -> Result<Vec<Reference>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            name, kind, hash, remote
        FROM
            refs
        WHERE
            kind = ?1 AND
            remote = ?2
        ",
    )?;
    statement
        .query_map(params![kind, remote], |row| {
            Ok(Reference {
                name: row.get(0)?,
                kind: row.get(1)?,
                hash: row.get(2)?,
                remote: row.get(3)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}

pub fn get(
    connection: &Connection,
    remote: Option<&str>,
    name: &str,
) -> Result<Reference, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            name,
            kind,
            hash,
            remote
        FROM
            refs
        WHERE
            name = ?1 AND
            remote = ?2
        ",
    )?;
    statement.query_row(params![name, remote], |row| {
        Ok(Reference {
            name: row.get(0)?,
            kind: row.get(1)?,
            hash: row.get(2)?,
            remote: row.get(3)?,
        })
    })
}
