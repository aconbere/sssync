use std::collections::HashSet;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models::commit::Commit;
use crate::models::tree_file::{TreeFile, TreeFileFileHash, TreeFilePathHash};

#[derive(Debug)]
pub struct TreeDiff {
    pub additions: Vec<TreeFile>,
    pub deletions: Vec<TreeFile>,
    pub changes: Vec<TreeFile>,
}

impl TreeDiff {
    pub fn empty() -> Self {
        Self {
            additions: vec![],
            deletions: vec![],
            changes: vec![],
        }
    }

    // The Problem:
    //
    // a: what files were added?
    // b: what files were deleted?
    // c: what files were changed?
    //
    // to do this we're going to first focus on file_hashes. We'll build a set
    // of file_hashes for the newer and older. Any file_hashes in newer but
    // not in older are either New or Changed. Any file_hashes in older but
    // not in newer are deletions.
    //
    // Now that we have new or chagned, we'll walk through the new or changed
    // files and look up if the path exists in the older. If the file is new
    // or changed, but the path exists in older then the file is changed,
    // otherwise it's new.
    //
    // Directionality is from older -> newer So for example if the newer set
    // contains a file that isn't found in the older. That file will end up
    // in the additions set.
    pub fn new(older: &Vec<TreeFile>, newer: &Vec<TreeFile>) -> Self {
        let older_by_hashes: HashSet<TreeFileFileHash> =
            older.iter().map(|f| TreeFileFileHash(f.clone())).collect();

        let newer_by_hashes: HashSet<TreeFileFileHash> =
            newer.iter().map(|f| TreeFileFileHash(f.clone())).collect();

        let older_by_path: HashSet<TreeFilePathHash> =
            older.iter().map(|f| TreeFilePathHash(f.clone())).collect();

        let mut additions = Vec::new();
        let mut changes_by_path = HashSet::new();

        // All new hashes found in the the new tree, files might either be
        // additions or changes. We can determine which by filtering the set of
        // additions by those that have matching paths in older.
        let all_positive_deltas: Vec<TreeFile> = newer_by_hashes
            .difference(&older_by_hashes)
            .map(|f| f.0.clone())
            .collect();

        for f in &all_positive_deltas {
            let path_hash = TreeFilePathHash(f.clone());
            if older_by_path.contains(&path_hash) {
                changes_by_path.insert(path_hash);
            } else {
                additions.push(f.clone());
            }
        }

        // Deletions are files in older state that can no longer be found in the
        // older state
        let deletions: Vec<TreeFile> = older_by_hashes
            .difference(&newer_by_hashes)
            .filter(|f| {
                let path_hash = TreeFilePathHash(f.0.clone());
                !changes_by_path.contains(&path_hash)
            })
            .map(|f| f.0.clone())
            .collect();

        let changes: Vec<TreeFile> =
            changes_by_path.into_iter().map(|f| f.0).collect();

        TreeDiff {
            additions,
            deletions,
            changes,
        }
    }

    pub fn updates(&self) -> Vec<TreeFile> {
        let mut updated_files: Vec<TreeFile> = self.additions.clone();
        updated_files.extend(self.changes.clone());
        updated_files
    }

    /* Adds two diffs together
     *
     * Operation is directionaly with other being applied "on top of" self.
     * operation will return an error if other tries to change a files
     * that self has deleted.
     */
    pub fn add(&self, other: &Self) -> Result<TreeDiff> {
        // additions
        // deletions
        // changes

        let additions_self = HashSet::from_iter(self.additions.clone());
        let deletions_self = HashSet::from_iter(self.deletions.clone());
        let changes_self = HashSet::from_iter(self.changes.clone());

        let additions_other = HashSet::from_iter(other.additions.clone());
        let deletions_other = HashSet::from_iter(other.deletions.clone());
        let changes_other = HashSet::from_iter(other.changes.clone());

        let deleted_changes: HashSet<TreeFile> = deletions_self
            .intersection(&changes_other)
            .cloned()
            .collect();

        if !deleted_changes.is_empty() {
            return Err(anyhow!("Later change was deleted in earlier diff"));
        }

        let changes: Vec<TreeFile> =
            changes_self.union(&changes_other).cloned().collect();

        let additions: Vec<TreeFile> = additions_self
            .union(&additions_other)
            .cloned()
            .collect::<HashSet<TreeFile>>()
            .difference(&deletions_other)
            .cloned()
            .collect();

        let deletions: Vec<TreeFile> = deletions_self
            .union(&deletions_other)
            .cloned()
            .collect::<HashSet<TreeFile>>()
            .difference(&additions_other)
            .cloned()
            .collect();

        Ok(TreeDiff {
            additions,
            deletions,
            changes,
        })
    }
}

pub fn diff_parent(
    connection: &Connection,
    commit: &Commit,
) -> Result<TreeDiff> {
    let tree = db::tree::get(&connection, &commit.hash)?;
    // If there is no parent hash then the tree is all there is
    if let Some(parent_hash) = &commit.parent_hash {
        let parent_tree = db::tree::get(&connection, &parent_hash)?;
        Ok(TreeDiff::new(&tree, &parent_tree))
    } else {
        let parent_tree: Vec<TreeFile> = Vec::new();
        Ok(TreeDiff::new(&tree, &parent_tree))
    }
}

// Note, I think this might actually need to collect up
// all of the previous files from all the previous commits
// into a big tree and then diff agains the most recent.
//
// But maybe that's for another day
pub fn diff(
    connection: &Connection,
    old_hash: &str,
    new_hash: &str,
) -> Result<TreeDiff> {
    let old_tree = db::tree::get(connection, old_hash)?;
    let new_tree = db::tree::get(connection, new_hash)?;
    Ok(TreeDiff::new(&old_tree, &new_tree))
}

pub fn diff_list(
    connection: &Connection,
    commits: &Vec<Commit>,
) -> Result<TreeDiff> {
    let head = commits.first().ok_or(anyhow!("no diff"))?;
    let parents: Vec<String> = commits
        .iter()
        .map(|c| c.parent_hash.clone())
        .flatten()
        .collect();

    // Fast forward commits
    let mut all_files: Vec<TreeFile> = Vec::new();

    for parent_hash in parents {
        let mut t = db::tree::get(connection, &parent_hash)?;
        all_files.append(&mut t);
    }

    let head_tree = db::tree::get(connection, &head.hash)?;

    Ok(TreeDiff::new(&all_files, &head_tree))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_empty_trees() -> Result<()> {
        let newer = Vec::new();
        let older = Vec::new();

        let result = TreeDiff::new(&newer, &older);
        let check = TreeDiff {
            additions: Vec::new(),
            deletions: Vec::new(),
            changes: Vec::new(),
        };

        assert_eq!(result.additions, check.additions);
        assert_eq!(result.deletions, check.deletions);
        assert_eq!(result.changes, check.changes);
        Ok(())
    }

    #[test]
    fn test_diff_newer_simple_addition() -> Result<()> {
        let file_a = TreeFile {
            path: String::from("path-a"),
            file_hash: String::from("hash-a"),
            size_bytes: 10,
            commit_hash: String::from("commit-a"),
        };
        let file_b = TreeFile {
            path: String::from("path-b"),
            file_hash: String::from("hash-b"),
            size_bytes: 10,
            commit_hash: String::from("commit-a"),
        };

        let newer = Vec::from([file_a.clone(), file_b.clone()]);
        let older = Vec::new();

        let result = TreeDiff::new(&older, &newer);
        assert!(result.deletions.is_empty());
        assert!(result.additions.contains(&file_a));
        assert!(result.additions.contains(&file_b));
        Ok(())
    }

    #[test]
    fn test_diff_change_in_a() -> Result<()> {
        // older
        let file_a = TreeFile {
            path: String::from("path-a"),
            file_hash: String::from("hash-a"),
            size_bytes: 10,
            commit_hash: String::from("commit-b"),
        };

        let file_b = TreeFile {
            path: String::from("path-b"),
            file_hash: String::from("hash-b"),
            size_bytes: 10,
            commit_hash: String::from("commit-b"),
        };

        //newer
        let file_a_prime = TreeFile {
            path: String::from("path-a"),
            file_hash: String::from("hash-a-prime"),
            size_bytes: 10,
            commit_hash: String::from("commit-a"),
        };
        let file_c = TreeFile {
            path: String::from("path-c"),
            file_hash: String::from("hash-c"),
            size_bytes: 10,
            commit_hash: String::from("commit-a"),
        };

        let older = Vec::from([file_a.clone(), file_b.clone()]);
        let newer = Vec::from([file_a_prime.clone(), file_c.clone()]);

        let result = TreeDiff::new(&older, &newer);
        let check = TreeDiff {
            changes: Vec::from([file_a_prime.clone()]),
            additions: Vec::from([file_c.clone()]),
            deletions: Vec::from([file_b.clone()]),
        };

        assert_eq!(result.changes, check.changes, "assert changes are found");
        assert_eq!(
            result.additions, check.additions,
            "assert additions are found"
        );
        assert_eq!(
            result.deletions, check.deletions,
            "assert deletions are found"
        );
        Ok(())
    }
}
