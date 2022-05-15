use std::error::Error;

use rusqlite;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::db::commit;
use crate::models::commit::Commit;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            refs (
                name TEXT PRIMARY KEY,
                hash TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, name: &str, hash: &str) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            refs (name, hash)
        VALUES
            (?1, ?2)
        ",
        params![name, hash],
    )?;
    Ok(())
}

pub fn update_head(connection: &Connection, hash: &str) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            refs (name, hash)
        VALUES
            (?1, ?2)
        ON CONFLICT(name)
        DO UPDATE
        SET
            hash = excluded.hash
        ",
        params!["HEAD", hash],
    )?;
    Ok(())
}

pub fn get_by_name(connection: &Connection, name: &str) -> Result<Option<Commit>, Box<dyn Error>> {
    match connection
        .query_row(
            "
        SELECT
            hash
        FROM
            refs
        WHERE
            name = ?1
        ",
            params![name],
            |row| {
                let h: String = row.get(0)?;
                Ok(h)
            },
        )
        .optional()
    {
        Ok(Some(hash)) => match commit::get(connection, &hash) {
            Ok(commit) => Ok(Some(commit)),
            Err(e) => Err(e.into()),
        },
        Ok(None) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn get_head(connection: &Connection) -> Result<Option<Commit>, Box<dyn Error>> {
    get_by_name(connection, "HEAD")
}
