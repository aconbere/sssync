use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::commit;
use crate::db;
use crate::file_entry::{all_files, compare_file_entry, FileEntry};
use crate::store;

pub fn add(
    connection: &Connection,
    full_path: &Path,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if full_path.is_dir() {
        println!("adding directory: {}", rel_path.display());
        let files = all_files(full_path).unwrap_or(vec![]);
        for file in files {
            let file_entry = FileEntry::hash(&root_path.join(&file), &file)?;
            println!("File: {}::{}", file_entry.path, file_entry.hash);

            fs::copy(
                root_path.join(&file_entry.path),
                store::object_path(root_path, &file_entry.hash),
            )?;

            db::staging::insert(connection, &file_entry)?;
        }

        return Ok(());
    }

    if full_path.is_file() {
        println!("adding file: {}", rel_path.display());
        let file_entry = FileEntry::hash(full_path, rel_path)?;
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
    let staged_files = db::staging::get_all(connection)?;

    let mut staged_map: HashMap<&str, &FileEntry> = HashMap::new();

    staged_files.iter().for_each(|fe| {
        staged_map.insert(fe.path.as_str(), fe);
    });

    let found_files = all_files(root_path)?;

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
            .for_each(|fe| match compare_file_entry(fe, root_path) {
                Ok(cp) => {
                    let state = if cp { "" } else { "modified: " };
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

    let hash = "";
    let commit = commit::new(&hash, "", "")?;
    db::commit::insert(connection, &commit)?;

    // for every staged file we want to copy them to the object store
    // with the filename representing their hash
    //
    // then we want to hash all the hashes and write that into a commit
    // in the db
    Ok(())
}
