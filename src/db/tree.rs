use rusqlite::params;
use rusqlite::Connection;

use std::error::Error;

use crate::models::tree_file::TreeFile;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            trees (
                path TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                size_bytes INTEGER NOT NULL,
                commit_hash TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, tree_entry: &TreeFile) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO trees (path, file_hash, size_bytes, commit_hash)
        VALUES (?1, ?2, ?3, ?4)
        ",
        params![
            tree_entry.path,
            tree_entry.file_hash,
            tree_entry.size_bytes,
            tree_entry.commit_hash
        ],
    )?;
    Ok(())
}

pub fn insert_batch(
    connection: &Connection,
    tree_entries: Vec<TreeFile>,
) -> Result<(), Box<dyn Error>> {
    for tree_entry in tree_entries {
        insert(connection, &tree_entry)?;
    }
    Ok(())
}

pub fn get_tree(connection: &Connection, hash: &str) -> Result<Vec<TreeFile>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "SELECT
            path, file_hash, size_bytes, commit_hash
        FROM
            trees
        WHERE
            commit_hash = ?1
        ",
    )?;

    statement
        .query_map(params![hash], |row| {
            Ok(TreeFile {
                path: row.get(0)?,
                file_hash: row.get(1)?,
                size_bytes: row.get(2)?,
                commit_hash: row.get(3)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}
