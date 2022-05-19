use std::error::Error;

use rusqlite::Connection;

use crate::db;
use crate::models::migration::Migration;
use crate::models::upload::Upload;

pub fn new(
    connection: &Connection,
    object_hashes: Vec<String>,
) -> Result<(Migration, Vec<Upload>), Box<dyn Error>> {
    let migration = Migration::new();
    db::migration::insert(connection, &migration)?;

    let uploads = object_hashes
        .iter()
        .map(|h| Upload::new(migration.id, &h))
        .collect();

    for upload in uploads {
        db::upload::insert(connection, &upload)?;
    }

    Ok((migration, uploads))
}

//pub fn run(migration) {}
