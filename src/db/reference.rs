use std::error::Error;

use rusqlite;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::reference::Kind;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    println!("reference::create_table");
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
    kind: &str,
    hash: &str,
) -> Result<(), Box<dyn Error>> {
    println!("reference::insert");
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
    println!("reference::update");
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
