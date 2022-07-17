use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::file;
use crate::models::staged_file;
use crate::models::tree_file;

pub fn status(connection: &Connection, root_path: &Path) -> Result<(), Box<dyn Error>> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?;

    //println!("Current Head: {}", head.hash);
    let tracked_files = match &head {
        Some(head) => {
            println!("Head: {}", head.hash);
            db::tree::get_tree(connection, &head.hash)?
        }
        None => Vec::new(),
    };

    let mut tracked_map: HashMap<&str, &tree_file::TreeFile> = HashMap::new();
    tracked_files.iter().for_each(|tf| {
        tracked_map.insert(tf.path.as_str(), tf);
    });

    println!("Tracked Files: {:?}", tracked_files);

    /* Fetch all staged files/
     */
    let staged_files = db::staging::get_all(connection)?;
    println!("Staged Files: {:?}", staged_files);

    let mut staged_map: HashMap<&str, &staged_file::StagedFile> = HashMap::new();

    staged_files.iter().for_each(|fe| {
        staged_map.insert(fe.path.as_str(), fe);
    });

    /* Fetch all files on disk
     *
     * Diff these files with the staged files and tracked files to fund untracked files.
     *
     * Of the files already tracked, we then need to rehash them (or look at other indicators) and
     * see if there are differences inside the file
     */
    let found_files = file::get_all(root_path)?;
    println!("Found Files: {:?}", found_files);

    let unstaged_files: Vec<&PathBuf> = found_files
        .iter()
        .filter(|path| match path.to_str() {
            Some(path) => !staged_map.contains_key(path) && !tracked_map.contains_key(path),
            None => false,
        })
        .collect();

    println!("Status\n");

    if staged_files.len() > 0 {
        println!("Staged Files");
        staged_files
            .iter()
            .for_each(|fe| match staged_file::compare_file_meta(fe, root_path) {
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
