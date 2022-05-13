use rusqlite::params;
use rusqlite::Connection;

use std::error::Error;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            commits (
                hash TEXT PRIMARY KEY,
                comment TEXT NOT NULL,
                author TEXT NOT NULL,
                created_unix_timestamp INTEGER NOT NULL,
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            commit (hash, comment, author, created_unix_timestamp)
        VALUES
            (?1, ?2, ?3, ?4)
        ",
        params![
            "asdfadfadf",
            "Tax Time",
            "anders@conbere.org<Anders Conbere>",
            12312,
        ],
    )?;
    Ok(())
}
