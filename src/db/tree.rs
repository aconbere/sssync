use rusqlite::params;
use rusqlite::Connection;

use std::error::Error;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            trees (
                path TEXT NOT NULL, file_hash TEXT NOT NULL, commit_hash TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            trees (
                path TEXT NOT NULL, file_hash TEXT NOT NULL, commit_hash TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}
