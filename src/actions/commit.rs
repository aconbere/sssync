use rusqlite::Connection;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::db;
use crate::models::commit::Commit;
use crate::models::file;
use crate::models::file::File;
use crate::models::reference::Kind;
use crate::models::tree_file::TreeFile;

fn join_files(set_a: &Vec<File>, set_b: &Vec<File>) -> Vec<File> {
    let mut result_set: HashSet<File> = HashSet::new();

    set_a.iter().for_each(|f| {
        result_set.insert(f.clone());
    });

    set_b.iter().for_each(|f| {
        result_set.insert(f.clone());
    });

    Vec::from_iter(result_set)
}

pub fn commit(
    connection: &Connection,
    root_path: &Path,
) -> Result<(), Box<dyn Error>> {
    /* It's possible that at this point the user has no commits in the
     * repository yet. We'll collapse that case down by returning
     * the empty vector
     */
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?;

    let tracked_files = match &head {
        Some(head) => db::tree::get(connection, &head.hash)?
            .iter()
            .map(|f| f.to_file())
            .collect(),
        None => Vec::new(),
    };

    let parent_hash = head.map(|h| h.hash);

    // Need to join staged and tracked files
    let staged_files: Vec<File> = db::staging::get_all(connection)?
        .iter()
        .map(|s| s.to_file())
        .collect();

    let result_files = join_files(&tracked_files, &staged_files);

    let hash = file::hash_all(&result_files);
    let commit = Commit::new(&hash, "", "", parent_hash)?;

    for f in &staged_files {
        let source = root_path.join(&f.path);
        let destination = root_path.join(".sssync/objects").join(&f.file_hash);
        println!(
            "copying staged file {} to {}",
            source.display(),
            destination.display()
        );
        fs::copy(source, destination)?;
    }

    match db::commit::insert(connection, &commit) {
        Err(e) => {
            println!("Nothing to commit: {}", e);
            return Ok(());
        }
        Ok(()) => {}
    }
    db::reference::update(connection, &meta.head, Kind::Branch, &commit.hash)?;
    db::staging::delete(connection)?;

    let tree_entries: Vec<TreeFile> = result_files
        .iter()
        .map(|fe| TreeFile::from_file(&commit.hash, fe))
        .collect();

    db::tree::insert_batch(connection, tree_entries)?;

    // for every staged file we want to copy them to the object store
    // with the filename representing their hash
    Ok(())
}
