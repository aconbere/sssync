use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

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
        UPDATE
            refs
        SET
            hash = ?1
        WHERE
            name = \"HEAD\"
        ",
        params![hash],
    )?;
    Ok(())
}
