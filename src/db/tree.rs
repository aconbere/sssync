use std::path::Path;

use anyhow::Result;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::tree_file::TreeFile;

/* A Tree represents a flattened file tree: A heirchal list of files, each
 * with a hash, a size in bytes, and a commit hash that connects them to the
 * rest of the sssync world.
 */
pub fn create_table(connection: &Connection) -> Result<()> {
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

pub fn insert(connection: &Connection, tree_entry: &TreeFile) -> Result<()> {
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
) -> Result<()> {
    for tf in tree_files {
        insert(connection, &tf)?;
    }
    Ok(())
}

pub fn get(connection: &Connection, hash: &str) -> Result<Vec<TreeFile>> {
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

    let result: Vec<TreeFile> = statement
        .query_map(params![hash], |row| {
            Ok(TreeFile {
                path: row.get(0)?,
                file_hash: row.get(1)?,
                size_bytes: row.get(2)?,
                commit_hash: row.get(3)?,
            })
        })?
        .into_iter()
        .flatten()
        .collect();
    Ok(result)
}

pub fn get_by_path(connection: &Connection, path: &Path) -> Result<TreeFile> {
    let mut statement = connection.prepare(
        "
        SELECT
            path, file_hash, size_bytes, commit_hash
        FROM
            trees
        WHERE
            path = ?1
        ",
    )?;

    let result: TreeFile =
        statement.query_row(params![path.to_str().unwrap()], |row| {
            Ok(TreeFile {
                path: row.get(0)?,
                file_hash: row.get(1)?,
                size_bytes: row.get(2)?,
                commit_hash: row.get(3)?,
            })
        })?;
    Ok(result)
}

pub fn get_all(
    connection: &Connection,
) -> Result<Vec<TreeFile>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            path, file_hash, size_bytes, commit_hash
        FROM
            trees
        ",
    )?;

    statement
        .query_map(params![], |row| {
            Ok(TreeFile {
                path: row.get(0)?,
                file_hash: row.get(1)?,
                size_bytes: row.get(2)?,
                commit_hash: row.get(3)?,
            })
        })
        .into_iter()
        .flatten()
        .collect()
}
