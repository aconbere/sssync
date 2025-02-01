use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::params;
use rusqlite::Connection;

use crate::models::commit::Commit;
use crate::models::tree_file::TreeFile;
use crate::tree::TreeDiff;

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

pub fn get_by_path(
    connection: &Connection,
    path: &Path,
) -> Result<TreeFile, rusqlite::Error> {
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

    statement.query_row(params![path.to_str().unwrap()], |row| {
        Ok(TreeFile {
            path: row.get(0)?,
            file_hash: row.get(1)?,
            size_bytes: row.get(2)?,
            commit_hash: row.get(3)?,
        })
    })
}

// Note, I think this might actually need to collect up
// all of the previous files from all the previous commits
// into a big tree and then diff agains the most recent.
//
// But maybe that's for another day
pub fn diff_commits(
    connection: &Connection,
    old_hash: &str,
    new_hash: &str,
) -> Result<TreeDiff> {
    let old_tree = get(connection, old_hash)?;
    let new_tree = get(connection, new_hash)?;
    Ok(TreeDiff::new(&old_tree, &new_tree))
}

pub fn diff_all_commits(
    connection: &Connection,
    commits: &Vec<Commit>,
) -> Result<TreeDiff> {
    let head = commits.first().ok_or(anyhow!("no diff"))?;
    let parents: Vec<String> = commits
        .iter()
        .map(|c| c.parent_hash.clone())
        .flatten()
        .collect();

    // Fast forward commits
    let mut all_files: Vec<TreeFile> = Vec::new();

    for parent_hash in parents {
        let mut t = get(connection, &parent_hash)?;
        all_files.append(&mut t);
    }

    let head_tree = get(connection, &head.hash)?;

    Ok(TreeDiff::new(&all_files, &head_tree))
}
