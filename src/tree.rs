use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::models::tree_file::{TreeFile, TreeFileFileHash, TreeFilePathHash};
use crate::store;
use anyhow::Result;

#[derive(Debug)]
pub struct TreeDiff {
    pub additions: Vec<TreeFile>,
    pub deletions: Vec<TreeFile>,
    pub changes: Vec<TreeFile>,
}

impl TreeDiff {
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

    pub fn apply(&self, root_path: &Path) -> Result<()> {
        for a in &self.additions {
            let destination = root_path.join(&a.path);
            store::export_to(root_path, &a.file_hash, &destination)?;
        }
        for d in &self.deletions {
            let destination = root_path.join(&d.path);
            fs::remove_file(destination)?;
        }
        Ok(())
    }
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
