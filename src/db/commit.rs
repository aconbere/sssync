use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::commit::Commit;

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

pub fn insert(connection: &Connection, commit: &Commit) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            commit (hash, comment, author, created_unix_timestamp)
        VALUES
            (?1, ?2, ?3, ?4)
        ",
        params![
            commit.hash,
            commit.comment,
            commit.author,
            commit.created_unix_timestamp,
        ],
    )?;
    Ok(())
}
