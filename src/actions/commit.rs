use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::models::commit;
use crate::models::file_entry;
use crate::models::tree_entry;

pub fn commit(connection: &Connection, root_path: &Path) -> Result<(), Box<dyn Error>> {
    let head = db::reference::get_head(connection)?;

    let tracked_files = match head {
        Some(head_commit) => {
            println!("Current Head: {}", head_commit.hash);
            db::tree::get_tree(connection, &head_commit.hash)?
        }
        None => Vec::new(),
    };
    // Need to join staged and tracked files

    let staged_files = db::staging::get_all(connection)?;

    // This is wrong; should be the concatenation of all
    // staged files overlayed with the files in the
    // current tree;
    let hash = file_entry::hash_all(&staged_files);
    let commit = commit::new(&hash, "", "")?;

    match db::commit::insert(connection, &commit) {
        Err(_) => {
            println!("Nothing to commit");
            return Ok(());
        }
        Ok(()) => {}
    }

    let tree_entries: Vec<tree_entry::TreeEntry> = staged_files
        .iter()
        .map(|fe| tree_entry::from_file_entry(&commit.hash, fe))
        .collect();

    db::tree::insert_batch(connection, tree_entries)?;

    // for every staged file we want to copy them to the object store
    // with the filename representing their hash
    //
    // then we want to hash all the hashes and write that into a commit
    // in the db
    Ok(())
}
