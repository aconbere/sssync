use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::migration::Migration;
use crate::models::upload::{Upload, UploadState};

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            uploads (
                migration_id TEXT NOT NULL,
                object_hash TEXT NOT NULL,
                state TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(
    connection: &Connection,
    upload: &Upload,
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            uploads (migration_id, object_hash, state)
        VALUES
            (?1, ?2, ?3)
        ",
        params![upload.migration_id, upload.object_hash, upload.state],
    )?;
    Ok(())
}

pub fn get_all_for_migration(
    connection: &Connection,
    migration: &Migration,
    state: UploadState,
) -> Result<Vec<Upload>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            migration_id, object_hash, state
        FROM
            uploads
        WHERE
            migration_id = ?1 AND
            state = ?2
        ",
    )?;
    statement
        .query_map(params![migration.id, state], |row| {
            Ok(Upload {
                migration_id: row.get(0)?,
                object_hash: row.get(1)?,
                state: row.get(2)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}

pub fn get_waiting_for_migration(
    connection: &Connection,
    migration: &Migration,
) -> Result<Vec<Upload>, rusqlite::Error> {
    get_all_for_migration(connection, migration, UploadState::Waiting)
}

pub fn set_state(
    connection: &Connection,
    upload: &Upload,
    state: UploadState,
) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        UPDATE
            uploads
        SET
            state = ?3
        WHERE
            migration_id = ?1 AND
            object_hash = ?2
        ",
        params![upload.migration_id, upload.object_hash, state],
    )?;
    Ok(())
}
