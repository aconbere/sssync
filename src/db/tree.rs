use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::commit::Commit;
use crate::models::tree_file::TreeFile;
use crate::tree::{diff_trees, DiffResult};

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
        "
        SELECT
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

pub fn diff(
    connection: &Connection,
    left: &Commit,
    right: &Commit,
) -> Result<DiffResult, Box<dyn Error>> {
    if left.hash == right.hash {
        return Ok(DiffResult {
            additions: vec![],
            deletions: vec![],
            changes: vec![],
        });
    }

    let left_tree = get_tree(connection, &left.hash)?;
    let right_tree = get_tree(connection, &left.hash)?;
    Ok(diff_trees(&left_tree, &right_tree))
}

// Go through each commit between two commits and add up all the added objects.
pub fn collected_additions(
    connection: &Connection,
    left: &Commit,
    right: &Commit,
) -> Result<(), Box<dyn Error>> {
    Ok(())
}
