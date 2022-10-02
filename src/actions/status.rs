use std::fmt;

use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::file;
use crate::models::index;
use crate::models::staged_file;
use crate::models::tree_file;

pub struct Status {
    index_files: Vec<index::Index>,
    unstaged_files: Vec<PathBuf>,
    deleted_files: Vec<tree_file::TreeFile>,
}

impl fmt::Display for Status {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "Changes to be committed:\n")?;
        for f in &self.index_files {
            match f {
                index::Index::DeletedFile(p) => {
                    write!(w, "\tdeleted: {}\n", p.path)?;
                }
                index::Index::ChangedFile(p) => {
                    write!(w, "\tmodified: {}\n", p.path)?;
                }
                index::Index::AddedFile(p) => {
                    write!(w, "\tadded: {}\n", p.path)?;
                }
            }
        }
        write!(w, "\n")?;

        if !self.unstaged_files.is_empty() {
            write!(w, "Changes not staged for commit:\n")?;
            for f in &self.unstaged_files {
                write!(w, "\tmodified: {}\n", f.to_str().unwrap())?;
            }
        }

        if !self.deleted_files.is_empty() {
            write!(w, "Deleted files:\n")?;
            for f in &self.deleted_files {
                write!(w, "\tdeleted: {}\n", f.path)?;
            }
        }

        Ok(())
    }
}

/* The goal of status is to compare three states:
 *  - The state of the store
 *  - The state of the index
 *  - The state of the filesystem
 *
 *  It does this by building up a set of each of these files (TreeFiles), and comparing
 *  the sets to produce a human readable string outpute.
 */
pub fn status(
    connection: &Connection,
    root_path: &Path,
) -> Result<Status, Box<dyn Error>> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or("no head")?;

    /* Tracked files are files that are already in the store. */
    let tracked_files = db::tree::get(connection, &head.hash)?;
    let mut tracked_map: HashMap<&str, &tree_file::TreeFile> = HashMap::new();
    tracked_files.iter().for_each(|tf| {
        tracked_map.insert(tf.path.as_str(), tf);
    });

    /* Fetch all files on disk */
    let disk_files = file::get_all(root_path)?;
    let mut disk_map: HashMap<&str, &PathBuf> = HashMap::new();
    disk_files.iter().for_each(|pb| {
        disk_map.insert(pb.to_str().unwrap(), pb);
    });

    /* Staged files have been added to the index */
    let staged_files = db::staging::get_all(connection)?;
    let mut index_files: Vec<index::Index> = Vec::new();
    let mut staged_map: HashMap<&str, staged_file::StagedFile> = HashMap::new();
    staged_files.iter().for_each(|se| {
        staged_map.insert(se.path.as_str(), se.clone());

        let full_path = root_path.join(&se.path);

        if se.compare_lstat(&full_path).unwrap_or(false) {
            index_files.push(index::Index::ChangedFile(se.clone()));
        } else if !disk_map.contains_key(se.path.as_str()) {
            index_files.push(index::Index::DeletedFile(se.clone()));
        } else {
            index_files.push(index::Index::AddedFile(se.clone()));
        }
    });

    /* Unstaged files are files on disk that are neither in the set of staged files
     * nor in the set of tracked files.
     *
     * Deleted files are files that are tracked, but not in the staging or on disk
     *
     * Staged files may be changed from the version on disk.
     *
     * Diff these files with the staged files and tracked files to fund untracked files.
     *
     * Of the files already tracked, we then need to rehash them (or look at other indicators) and
     * see if there are differences inside the file
     */
    let mut unstaged_files: Vec<PathBuf> = Vec::new();
    disk_files.iter().for_each(|df| {
        if let Some(p) = df.to_str() {
            if !staged_map.contains_key(p) && !tracked_map.contains_key(p) {
                unstaged_files.push(df.clone())
            }
        }
    });

    let mut deleted_files: Vec<tree_file::TreeFile> = Vec::new();
    for tf in tracked_files {
        if !staged_map.contains_key(tf.path.as_str())
            && !disk_map.contains_key(tf.path.as_str())
        {
            deleted_files.push(tf)
        }
    }

    Ok(Status {
        index_files: index_files,
        unstaged_files: unstaged_files,
        deleted_files: deleted_files,
    })
}
