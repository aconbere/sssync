use rusqlite::Connection;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::db;
use crate::models::commit::Commit;
use crate::models::reference::Kind;
use crate::models::staged_file::Change;
use crate::models::status::{
    hash_all, intermediate_to_tree_files, Hashable, IntermediateTree, Status,
};
use crate::store;

pub fn commit(
    connection: &Connection,
    root_path: &Path,
    message: &str,
) -> Result<()> {
    let staged_files = db::staging::get_all(connection)?;
    if staged_files.is_empty() {
        return Err(anyhow!("Staging is empty: Nothing to commit"));
    }

    let status = Status::new(connection, root_path)?;
    let parent_hash = status.head.map(|h| h.hash);

    let staged_changes = db::staging::get_all(connection)?;

    // This map will end up collecting together both the tracked set of files as
    // well as the staged files. Then we can add or remove files by path as
    // we work through staged additions and deletions.
    //
    // IntermediatTree is just an enum that let's us work on top of two types of
    // files with slightly different data. (Note: I should put these all
    // together to make them easier to work on)
    //
    // NOTE! These tree files will have the commit hash of the parent commit
    // They need to have that updated before saving them or they will point
    // to the wrong commit.
    let mut new_tree: HashMap<PathBuf, IntermediateTree> = status
        .tracked_files
        .iter()
        .map(|(pb, tf)| (pb.clone(), IntermediateTree::Committed(tf.clone())))
        .collect();

    for a in staged_changes {
        match a {
            Change::Addition(sf) => {
                store::insert_from(
                    root_path,
                    &sf.file_hash,
                    &root_path.join(&sf.path),
                )?;
                new_tree.insert(
                    PathBuf::from(sf.path.clone()),
                    IntermediateTree::Staged(sf.clone()),
                );
            }
            Change::Deletion(pb) => {
                new_tree.remove(&pb);
            }
        }
    }

    let tree_files: Vec<IntermediateTree> =
        new_tree.clone().into_values().collect();

    let hashable_tree_files: Vec<Box<dyn Hashable>> = new_tree
        .into_iter()
        .map(|(_, t)| {
            let i: Box<dyn Hashable> = Box::new(t);
            i
        })
        .collect();

    let hash = hash_all(&hashable_tree_files);
    let commit = Commit::new(&hash, message, "", parent_hash)?;

    db::commit::insert(connection, &commit)?;
    db::reference::update(
        connection,
        &status.ref_name,
        Kind::Branch,
        &commit.hash,
    )?;
    db::staging::delete(connection)?;

    db::tree::insert_batch(
        connection,
        intermediate_to_tree_files(&tree_files, &commit.hash),
    )?;

    // for every staged file we want to copy them to the object store
    // with the filename representing their hash
    Ok(())
}
