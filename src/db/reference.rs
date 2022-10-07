use std::error::Error;

use rusqlite;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::reference::{Kind, Reference};

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            refs (
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                hash TEXT NOT NULL,
                PRIMARY KEY (name, kind)
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
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            refs (name, kind, hash)
        VALUES
            (?1, ?2, ?3)
        ",
        params![name, kind, hash],
    )?;
    Ok(())
}

pub fn update(
    connection: &Connection,
    name: &str,
    kind: Kind,
    hash: &str,
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            refs (name, kind, hash)
        VALUES
            (?1, ?2, ?3)
        ON CONFLICT(name, kind)
        DO UPDATE
        SET
            hash = excluded.hash
        ",
        params![name, kind, hash],
    )?;
    Ok(())
}

pub fn get_all_by_kind(
    connection: &Connection,
    kind: Kind,
) -> Result<Vec<Reference>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            name, kind, hash
        FROM
            refs
        WHERE
            kind = ?1
        ",
    )?;
    statement
        .query_map(params![kind], |row| {
            Ok(Reference {
                name: row.get(0)?,
                kind: row.get(1)?,
                hash: row.get(2)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}

pub fn get(
    connection: &Connection,
    name: &str,
) -> Result<Reference, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            name,
            kind,
            hash
        FROM
            refs
        WHERE
            name = ?1
        ",
    )?;
    statement.query_row(params![name], |row| {
        Ok(Reference {
            name: row.get(0)?,
            kind: row.get(1)?,
            hash: row.get(2)?,
        })
    })
}
