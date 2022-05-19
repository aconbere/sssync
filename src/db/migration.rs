use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::migration::Migration;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            migrations (
                id TEXT PRIMARY KEY,
                state TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, migration: &Migration) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            migrations (id, state)
        VALUES
            (?1, ?2)
        ",
        params![migration.id, migration.state],
    )?;
    Ok(())
}
