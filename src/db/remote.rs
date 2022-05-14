use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::remote::Remote;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            remotes (
                name PRIMARY KEY,
                url NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, remote: &Remote) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO remotes (name, url)
        VALUES (?1, ?2)
        ",
        params![remote.name, String::from(remote.url.clone())],
    )?;
    Ok(())
}
