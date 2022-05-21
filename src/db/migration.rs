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
                kind TEXT NOT NULL
                remote_name TEXT NOT NULL
                remote_kind TEXT NOT NULL
                remote_location TEXT NOT NULL
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
            migrations (id, kind, remote_name, remote_kind, remote_location, state)
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6)
        ",
        params![
            migration.id,
            migration.kind,
            migration.remote_name,
            migration.remote_kind,
            migration.remote_location,
            migration.state
        ],
    )?;
    Ok(())
}
