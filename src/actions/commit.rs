use rusqlite::Connection;
use std::collections::HashSet;
use std::error::Error;

use crate::db;
use crate::models::commit::Commit;
use crate::models::file;
use crate::models::file::File;
use crate::models::tree_file::TreeFile;

fn join_files(set_a: Vec<File>, set_b: Vec<File>) -> Vec<File> {
    let mut result_set: HashSet<File> = HashSet::new();

    set_a.iter().for_each(|f| {
        result_set.insert(f.clone());
    });

    set_b.iter().for_each(|f| {
        result_set.insert(f.clone());
    });

    Vec::from_iter(result_set)
}

pub fn commit(connection: &Connection) -> Result<(), Box<dyn Error>> {
    /* It's possible that at this point the user has no commits in the
     * repository yet. We'll collapse that case down by returning
     * the empty vector
     */
    let head = db::reference::get_head(connection)?;

    let tracked_files: Vec<File> = match &head {
        Some(head_commit) => {
            println!("Current Head: {}", head_commit.hash);
            db::tree::get_tree(connection, &head_commit.hash)?
        }
        None => Vec::new(),
    }
    .iter()
    .map(|f| f.to_file())
    .collect();

    // Need to join staged and tracked files
    let staged_files: Vec<File> = db::staging::get_all(connection)?
        .iter()
        .map(|s| s.to_file())
        .collect();

    let result_files = join_files(tracked_files, staged_files);

    let hash = file::hash_all(&result_files);
    let parent_hash = head.map(|h| h.hash);
    let commit = Commit::new(&hash, "", "", parent_hash)?;

    match db::commit::insert(connection, &commit) {
        Err(e) => {
            println!("Nothing to commit: {}", e);
            return Ok(());
        }
        Ok(()) => {}
    }

    let tree_entries: Vec<TreeFile> = result_files
        .iter()
        .map(|fe| TreeFile::from_file(&commit.hash, fe))
        .collect();

    db::tree::insert_batch(connection, tree_entries)?;

    // for every staged file we want to copy them to the object store
    // with the filename representing their hash
    //
    // then we want to hash all the hashes and write that into a commit
    // in the db
    Ok(())
}
