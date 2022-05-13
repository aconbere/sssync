use rusqlite::params;
use rusqlite::Connection;

use std::error::Error;

use crate::models::tree_entry::TreeEntry;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            trees (
                path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                commit_hash TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, tree_entry: &TreeEntry) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO trees (path, file_hash, commit_hash)
        VALUES (?1, ?2, ?3)
        ",
        params![
            tree_entry.path,
            tree_entry.file_hash,
            tree_entry.commit_hash
        ],
    )?;
    Ok(())
}

pub fn insert_batch(
    connection: &Connection,
    tree_entries: Vec<TreeEntry>,
) -> Result<(), Box<dyn Error>> {
    for tree_entry in tree_entries {
        insert(connection, &tree_entry)?;
    }
    Ok(())
}
