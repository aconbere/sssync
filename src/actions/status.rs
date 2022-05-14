use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::file_entry;

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
