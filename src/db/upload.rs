use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::upload::Upload;

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

pub fn insert(connection: &Connection, upload: &Upload) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO
            uploades (migration_id, object_hash, state)
        VALUES
            (?1, ?2, ?3)
        ",
        params![upload.migration_id, upload.object_hash, upload.state],
    )?;
    Ok(())
}
