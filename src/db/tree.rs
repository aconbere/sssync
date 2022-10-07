use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::commit::Commit;
use crate::models::tree_file::TreeFile;
use crate::tree;

/* A Tree represents a flattened file tree: A heirchal list of files, each with a hash, a size in
 * bytes, and a commit hash that connects them to the rest of the sssync world.
 */
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

pub fn insert(
    connection: &Connection,
    tree_entry: &TreeFile,
) -> Result<(), Box<dyn Error>> {
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
    tree_files: Vec<TreeFile>,
) -> Result<(), Box<dyn Error>> {
    for tf in tree_files {
        insert(connection, &tf)?;
    }
    Ok(())
}

pub fn get(
    connection: &Connection,
    hash: &str,
) -> Result<Vec<TreeFile>, rusqlite::Error> {
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
    older: &Commit,
    newer: &Commit,
) -> Result<tree::TreeDiff, Box<dyn Error>> {
    if older.hash == newer.hash {
        println!("db::tree::diff no diff");
        return Ok(tree::TreeDiff {
            additions: vec![],
            deletions: vec![],
            changes: vec![],
        });
    }

    let older_tree = get(connection, &older.hash)?;
    let newer_tree = get(connection, &newer.hash)?;
    Ok(tree::diff(&older_tree, &newer_tree))
}
