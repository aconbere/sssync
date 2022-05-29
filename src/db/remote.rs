use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::remote::Remote;
use crate::types::remote_kind::RemoteKind;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            remotes (
                name PRIMARY KEY,
                kind TEXT NOT NULL,
                location TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, remote: &Remote) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO remotes (name, kind, location)
        VALUES (?1, ?2, ?3)
        ",
        params![remote.name, remote.kind, remote.location],
    )?;
    Ok(())
}

struct IntermediateRemote {
    name: String,
    kind: RemoteKind,
    location: String,
}

pub fn get_all(connection: &Connection) -> Result<Vec<Remote>, Box<dyn Error>> {
    let inter = get_all_intermediate(connection)?;
    inter
        .iter()
        .map(|e| Remote::new(&e.name, e.kind, &e.location))
        .collect()
}

fn get_all_intermediate(
    connection: &Connection,
) -> Result<Vec<IntermediateRemote>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            name, kind, location
        FROM
            remotes
        ",
    )?;

    statement
        .query_map(params![], |row| {
            Ok(IntermediateRemote {
                name: row.get(0)?,
                kind: row.get(1)?,
                location: row.get(2)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}

pub fn get(connection: &Connection, name: &str) -> Result<Remote, Box<dyn Error>> {
    let inter = get_intermediate(connection, name)?;
    Ok(Remote::new(&inter.name, inter.kind, &inter.location)?)
}

fn get_intermediate(
    connection: &Connection,
    name: &str,
) -> Result<IntermediateRemote, rusqlite::Error> {
    connection.query_row(
        "
        SELECT
            name, kind, location
        FROM
            remotes
        WHERE
            name = ?1
        ",
        params![name],
        |row| {
            Ok(IntermediateRemote {
                name: row.get(0)?,
                kind: row.get(1)?,
                location: row.get(2)?,
            })
        },
    )
}

pub fn delete(connection: &Connection, name: &str) -> Result<(), rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        DELETE
        FROM
            remotes
        WHERE
            name = ?1
        ",
    )?;

    statement.execute(params![name])?;
    Ok(())
}
