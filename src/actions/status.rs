use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::file_entry;
use crate::models::tree_entry;

pub fn status(connection: &Connection, root_path: &Path) -> Result<(), Box<dyn Error>> {
    let head = db::reference::get_head(connection)?;

    /* Fetch all tracked files/
     */
    let tracked_files = match head {
        Some(head_commit) => {
            println!("Current Head: {}", head_commit.hash);
            db::tree::get_tree(connection, &head_commit.hash)?
        }
        None => Vec::new(),
    };

    let mut tracked_map: HashMap<&str, &tree_entry::TreeEntry> = HashMap::new();
    tracked_files.iter().for_each(|tf| {
        tracked_map.insert(tf.path.as_str(), tf);
    });

    /* Fetch all staged files/
     */
    let staged_files = db::staging::get_all(connection)?;

    let mut staged_map: HashMap<&str, &file_entry::FileEntry> = HashMap::new();

    staged_files.iter().for_each(|fe| {
        staged_map.insert(fe.path.as_str(), fe);
    });

    /* Fetch all files on disk
     *
     * Diff these files with the staged files and
     * tracked files to fund untracked files.
     */
    let found_files = file_entry::get_all(root_path)?;

    let unstaged_files: Vec<&PathBuf> = found_files
        .iter()
        .filter(|path| match path.to_str() {
            Some(s) => !staged_map.contains_key(s) && !tracked_map.contains_key(s),
            None => false,
        })
        .collect();

    println!("Status\n");

    if staged_files.len() > 0 {
        println!("Staged Files");
        staged_files
            .iter()
            .for_each(|fe| match file_entry::compare_file_meta(fe, root_path) {
                Ok(cp) => {
                    let state = if !cp { "" } else { "modified: " };
                    println!("\t{}{}", state, fe.path)
                }
                Err(_) => {}
            });
    }

    if unstaged_files.len() > 0 {
        println!("Untracked Files");
        unstaged_files.iter().for_each(|p| match p.to_str() {
            Some(s) => println!("\t{}", s),
            None => {}
        });
    }

    Ok(())
}
