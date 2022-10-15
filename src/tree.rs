use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::models::tree_file::TreeFile;
use crate::store;

#[derive(Debug, PartialEq)]
pub struct TreeDiff {
    pub additions: HashSet<TreeFile>,
    pub deletions: HashSet<TreeFile>,
}

impl TreeDiff {
    // The Problem:
    //
    // a: what files were added?
    // b: what files were deleted?
    // c: what files were changed?
    //
    // to do this we're going to first focus on file_hashes. We'll build a set of file_hashes for the
    // newer and older. Any file_hashes in newer but not in older are either New or Changed. Any
    // file_hashes in older but not in newer are deletions.
    //
    // Now that we have new or chagned, we'll walk through the new or changed files and look up if the
    // path exists in the older. If the file is new or changed, but the path exists in older then the
    // file is changed, otherwise it's new.
    //
    // Directionality is from older -> newer So for example if the newer set contains a file that isn't
    // found in the older. That file will end up in the additions set.
    pub fn new(older: &HashSet<TreeFile>, newer: &HashSet<TreeFile>) -> Self {
        // Additions are files in the more recent state that can't be found in the older state
        let additions: HashSet<TreeFile> =
            newer.difference(older).cloned().collect();

        // Deletions are files in older state that can no longer be found in the older state
        let deletions: HashSet<TreeFile> =
            older.difference(newer).cloned().collect();

        TreeDiff {
            additions,
            deletions,
        }
    }

    pub fn apply(&self, root_path: &Path) -> Result<(), Box<dyn Error>> {
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
    use std::error::Error;

    #[test]
    fn test_diff_empty_trees() -> Result<(), Box<dyn Error>> {
        let newer = HashSet::new();
        let older = HashSet::new();

        let result = TreeDiff::new(&newer, &older);
        assert_eq!(
            result,
            TreeDiff {
                additions: HashSet::new(),
                deletions: HashSet::new(),
            }
        );
        Ok(())
    }

    #[test]
    fn test_diff_newer_simple_addition() -> Result<(), Box<dyn Error>> {
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

        let newer = HashSet::from([file_a.clone(), file_b.clone()]);
        let older = HashSet::new();

        let result = TreeDiff::new(&older, &newer);
        assert!(result.deletions.is_empty());
        assert!(result.additions.contains(&file_a));
        assert!(result.additions.contains(&file_b));
        Ok(())
    }

    #[test]
    fn test_diff_change_in_a() -> Result<(), Box<dyn Error>> {
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

        let file_a_prime = TreeFile {
            path: String::from("path-a"),
            file_hash: String::from("hash-a-prime"),
            size_bytes: 10,
            commit_hash: String::from("commit-a"),
        };

        let newer = HashSet::from([file_a.clone(), file_b.clone()]);
        let older = HashSet::from([file_a_prime.clone()]);

        let result = TreeDiff::new(&older, &newer);

        println!("additions: {:#?}", result.additions);
        println!("deletions: {:#?}", result.deletions);

        assert_eq!(
            result.additions,
            HashSet::from([file_a.clone(), file_b.clone()])
        );
        assert_eq!(result.deletions, HashSet::from([file_a_prime.clone()]));
        Ok(())
    }
}
