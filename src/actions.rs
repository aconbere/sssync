use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::commit;
use crate::models::file_entry;
use crate::models::tree_entry;
use crate::store;

pub fn add(
    connection: &Connection,
    full_path: &Path,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if full_path.is_dir() {
        println!("adding directory: {}", rel_path.display());
        let files = file_entry::get_all(full_path).unwrap_or(vec![]);
        for file in files {
            let file_entry = file_entry::FileEntry::hash(&root_path.join(&file), &file)?;
            println!("File: {}::{}", file_entry.path, file_entry.hash);

            file_entry::copy_if_not_present(&file_entry, root_path)?;
            db::staging::insert(connection, &file_entry)?;
        }

        return Ok(());
    }

    if full_path.is_file() {
        println!("adding file: {}", rel_path.display());
        let file_entry = file_entry::FileEntry::hash(full_path, rel_path)?;
        println!("File: {}::{}", file_entry.path, file_entry.hash);
        fs::copy(
            root_path.join(&file_entry.path),
            store::object_path(root_path, &file_entry.hash),
        )?;
        db::staging::insert(connection, &file_entry)?;
        return Ok(());
    }

    Ok(())
}

pub fn status(connection: &Connection, root_path: &Path) -> Result<(), Box<dyn Error>> {
    let head = db::reference::get_head(connection)?;
    println!("WTF");

    match head {
        Some(head_commit) => {
            println!("Current Head: {}", head_commit.hash);
        }
        None => {}
    }

    let staged_files = db::staging::get_all(connection)?;

    let mut staged_map: HashMap<&str, &file_entry::FileEntry> = HashMap::new();

    staged_files.iter().for_each(|fe| {
        staged_map.insert(fe.path.as_str(), fe);
    });

    let found_files = file_entry::get_all(root_path)?;

    let unstaged_files: Vec<&PathBuf> = found_files
        .iter()
        .filter(|path| match path.to_str() {
            Some(s) => !staged_map.contains_key(s),
            None => false,
        })
        .collect();

    println!("Status\n");

    if staged_files.len() > 0 {
        println!("Staged Files");
        staged_files
            .iter()
            .for_each(|fe| match file_entry::compare_to_disk(fe, root_path) {
                Ok(cp) => {
                    let state = if !cp { "" } else { "modified: " };
                    println!("\t{}{}", state, fe.path)
                }
                Err(_) => {}
            });
    }

    if unstaged_files.len() > 0 {
        println!("Unstaged Files");
        unstaged_files.iter().for_each(|p| match p.to_str() {
            Some(s) => println!("\t{}", s),
            None => {}
        });
    }

    Ok(())
}

pub fn init(root_path: &Path) -> Result<(), Box<dyn Error>> {
    println!("init: {}", root_path.display());
    store::init(&root_path)?;

    let connection = db::get_connection(&root_path)?;
    println!("found connection: {}", root_path.display());
    db::init(&connection)?;
    Ok(())
}

pub fn commit(connection: &Connection, root_path: &Path) -> Result<(), Box<dyn Error>> {
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
