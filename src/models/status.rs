use std::fmt;

use crate::models::staged_file::Change;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;
use crate::models::commit::Commit;
use crate::models::file;
use crate::models::staged_file::StagedFile;
use crate::models::tree_file::TreeFile;

use crate::hash::hash_string;

#[derive(Clone, Debug)]
pub enum IntermediateTree {
    Staged(StagedFile),
    Committed(TreeFile),
}

pub fn intermediate_to_tree_files(
    files: &Vec<&IntermediateTree>,
    commit_hash: &str,
) -> Vec<TreeFile> {
    files
        .into_iter()
        .map(|i_f| match i_f {
            IntermediateTree::Staged(sf) => sf.to_tree_file(&commit_hash),
            IntermediateTree::Committed(tf) => tf.clone(),
        })
        .collect()
}

pub fn hash_all(files: &Vec<&IntermediateTree>) -> String {
    let hashes: Vec<&str> = files
        .into_iter()
        .map(|f| match f {
            IntermediateTree::Staged(sf) => sf.file_hash.as_str(),
            IntermediateTree::Committed(tf) => tf.file_hash.as_str(),
        })
        .collect();

    hash_string(hashes.join(""))
}

pub struct Status {
    /* The set of files tracked at HEAD
     */
    pub tracked_files: HashMap<PathBuf, TreeFile>,
    /* Staged changes can be either additions or deletions:
     *
     * For ease of use later we'll move them into hash sets for both additions and deletions.
     */
    pub staged_additions: HashSet<PathBuf>,
    pub staged_deletions: HashSet<PathBuf>,

    /* A staged addition could potentially have changed or been deleted since it's addition to
     * the index. Conversely a staged deletion could have the file appear again.
     */
    pub staged_but_changed: HashSet<PathBuf>,
    pub staged_but_deleted: HashSet<PathBuf>,
    pub staged_but_added: HashSet<PathBuf>,

    /* Unstaged additions are files on disk that are neither in the set of staged additions
     * nor in the set of tracked files.
     */
    pub unstaged_additions: Vec<PathBuf>,

    /* Unstaged deletions are files deleted from disk that are in the set of tracked files, but
     * not in the set of staged deletions.
     */
    pub unstaged_deletions: Vec<PathBuf>,

    /* The current commit */
    pub head: Option<Commit>,
    /* The current ref name */
    pub ref_name: String,
}

impl fmt::Display for Status {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.staged_additions.is_empty() {
            write!(w, "Files staged to be added:\n")?;
            for f in &self.staged_additions {
                if self.staged_but_changed.contains(f) {
                    write!(w, "\tmodified: {}\n", f.to_str().unwrap())?;
                } else if self.staged_but_deleted.contains(f) {
                    write!(w, "\tdeleted: {}\n", f.to_str().unwrap())?;
                } else {
                    write!(w, "\tadded: {}\n", f.to_str().unwrap())?;
                }
            }
        }

        if !self.staged_deletions.is_empty() {
            write!(w, "Files staged to be deleted:\n")?;
            for f in &self.staged_deletions {
                if self.staged_but_added.contains(f) {
                    write!(w, "\tadded: {}\n", f.to_str().unwrap())?;
                } else {
                    write!(w, "\tdeleted: {}\n", f.to_str().unwrap())?;
                }
            }
        }

        if !self.unstaged_additions.is_empty() {
            write!(w, "Unstaged additions:\n")?;
            for f in &self.unstaged_additions {
                write!(w, "\tmodified: {}\n", f.to_str().unwrap())?;
            }
        }

        if !self.unstaged_deletions.is_empty() {
            write!(w, "Unstaged deletions:\n")?;
            for f in &self.unstaged_deletions {
                write!(w, "\tdeleted: {}\n", f.to_str().unwrap())?;
            }
        }

        Ok(())
    }
}

impl Status {
    /* The goal of status is to compare three states:
     *  - The state of the store
     *  - The state of the index
     *  - The state of the filesystem
     *
     *  It does this by building up a set of each of these files (TreeFiles), and comparing
     *  the sets to produce a human readable string outpute.
     */
    pub fn new(
        connection: &Connection,
        root_path: &Path,
    ) -> Result<Status, Box<dyn Error>> {
        let meta = db::meta::get(connection)?;
        let head = db::commit::get_by_ref_name(connection, &meta.head)?;

        /* Tracked files are files that are already in the store. */
        let tracked_files: HashMap<PathBuf, TreeFile> = match &head {
            Some(head) => HashMap::from_iter(
                db::tree::get(connection, &head.hash)?
                    .iter()
                    .map(|tf| (PathBuf::from(tf.path.clone()), tf.clone())),
            ),
            None => HashMap::new(),
        };

        /* Fetch all files on disk */
        let disk_files: HashSet<PathBuf> =
            HashSet::from_iter(file::get_all(root_path)?);

        /* Staged changes can be either additions or deletions:
         *
         * For ease of use later we'll move them into hash sets for both additions and deletions.
         */
        let staged_changes = db::staging::get_all(connection)?;
        let mut staged_additions: HashSet<PathBuf> = HashSet::new();
        let mut staged_deletions: HashSet<PathBuf> = HashSet::new();

        /* A staged addition could potentially have changed or been deleted since it's addition to
         * the index. Conversely a staged deletion could have the file appear again.
         */
        let mut staged_but_changed: HashSet<PathBuf> = HashSet::new();
        let mut staged_but_deleted: HashSet<PathBuf> = HashSet::new();
        let mut staged_but_added: HashSet<PathBuf> = HashSet::new();

        /* Unstaged additions are files on disk that are neither in the set of staged additions
         * nor in the set of tracked files.
         */
        let mut unstaged_additions: Vec<PathBuf> = Vec::new();

        /* Unstaged deletions are files deleted from disk that are in the set of tracked files, but
         * not in the set of staged deletions.
         */
        let mut unstaged_deletions: Vec<PathBuf> = Vec::new();

        staged_changes.iter().for_each(|sc| match sc {
            Change::Addition(sf) => {
                let path = PathBuf::from(&sf.path);
                staged_additions.insert(path.clone());

                let full_path = root_path.join(&sf.path);

                if sf.compare_lstat(&full_path).unwrap_or(false) {
                    staged_but_changed.insert(path.clone());
                } else if !disk_files.contains(&path) {
                    staged_but_deleted.insert(path);
                }
            }
            Change::Deletion(pb) => {
                staged_deletions.insert(pb.clone());

                if disk_files.contains(pb) {
                    staged_but_added.insert(pb.clone());
                }
            }
        });

        // Note - bug - should compare the lstat of df against the tracked
        // file if the tracked file exists.
        disk_files.iter().for_each(|df| {
            if !staged_additions.contains(df) {
                if !tracked_files.contains_key(df) {
                    unstaged_additions.push(df.clone());
                    return;
                }

                // at this point I need a way to cheaply compare
                // a tracked file and a disk file. Staged files
                // contain a modified time, but tracked files don't
                // given that, I think the best we can do here is
                // checking the size_bytes? Or kick off an expensive
                // hash?
                //
                //
                // Maybe I should consider a cheapper hash like md5
                // just as a backup for situations like this?
                let tf = tracked_files.get(df).unwrap();

                let meta = file::lstat(df);

                if meta.is_err() {
                    return;
                }

                if tf.size_bytes != meta.unwrap().st_size {
                    unstaged_additions.push(df.clone());
                }
            }
        });

        tracked_files.iter().for_each(|(pb, _)| {
            if !disk_files.contains(pb) && !staged_deletions.contains(pb) {
                unstaged_deletions.push(pb.clone())
            }
        });

        Ok(Status {
            tracked_files: tracked_files,
            staged_additions: staged_additions,
            staged_deletions: staged_deletions,
            staged_but_changed: staged_but_changed,
            staged_but_deleted: staged_but_deleted,
            staged_but_added: staged_but_added,
            unstaged_additions: unstaged_additions,
            unstaged_deletions: unstaged_deletions,
            head: head,
            ref_name: meta.head,
        })
    }
}
