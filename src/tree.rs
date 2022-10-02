use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::models::tree_file::TreeFile;
use crate::store;

#[derive(Debug, PartialEq)]
pub struct DiffResult {
    pub additions: Vec<TreeFile>,
    pub deletions: Vec<TreeFile>,
    pub changes: Vec<TreeFile>,
}

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
pub fn diff(older: &Vec<TreeFile>, newer: &Vec<TreeFile>) -> DiffResult {
    let mut newer_set: HashMap<String, TreeFile> = HashMap::new();

    for tree_file in newer {
        newer_set.insert(tree_file.file_hash.clone(), tree_file.clone());
    }

    let mut older_set: HashMap<String, TreeFile> = HashMap::new();
    let mut older_paths: HashSet<String> = HashSet::new();

    for tree_file in older {
        older_set.insert(tree_file.file_hash.clone(), tree_file.clone());
        older_paths.insert(tree_file.path.clone());
    }

    let mut new_or_changed: Vec<TreeFile> = vec![];

    for tree_file in newer_set.values() {
        if !older_set.contains_key(&tree_file.file_hash) {
            new_or_changed.push(tree_file.clone());
        }
    }

    let mut changes = vec![];
    let mut additions = vec![];

    for tree_file in new_or_changed {
        if older_paths.contains(&tree_file.path) {
            changes.push(tree_file.clone());
        } else {
            additions.push(tree_file.clone());
        }
    }

    let mut deletions: Vec<TreeFile> = vec![];
    for tree_file in older_set.values() {
        if !newer_set.contains_key(&tree_file.file_hash) {
            deletions.push(tree_file.clone());
        }
    }

    DiffResult {
        additions: additions,
        deletions: deletions,
        changes: changes,
    }
}

pub fn apply_diff(root_path: &Path, diff: &DiffResult) -> Result<(), Box<dyn Error>> {
    for a in &diff.additions {
        let destination = root_path.join(&a.path);
        store::copy_object(root_path, &a.file_hash, &destination)?;
    }
    for d in &diff.deletions {
        let destination = root_path.join(&d.path);
        fs::remove_file(destination)?;
    }
    for c in &diff.changes {
        let destination = root_path.join(&c.path);
        store::copy_object(root_path, &c.file_hash, &destination)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_diff_empty_trees() -> Result<(), Box<dyn Error>> {
        let newer = vec![];
        let older = vec![];

        let result = diff(&newer, &older);
        assert_eq!(
            result,
            DiffResult {
                additions: vec![],
                deletions: vec![],
                changes: vec![],
            }
        );
        Ok(())
    }

    #[test]
    fn test_diff_newer_simple_addition() -> Result<(), Box<dyn Error>> {
        let file_a = TreeFile::new("path-a", "hash-a", 10, "commit-a");
        let file_b = TreeFile::new("path-b", "hash-b", 10, "commit-a");

        let newer = vec![file_a.clone(), file_b.clone()];
        let older = vec![];

        let result = diff(&newer, &older);
        assert!(result.deletions.is_empty());
        assert!(result.changes.is_empty());
        assert!(result.additions.contains(&file_a));
        assert!(result.additions.contains(&file_b));
        Ok(())
    }

    #[test]
    fn test_diff_change_in_a() -> Result<(), Box<dyn Error>> {
        let file_a = TreeFile::new("path-a", "hash-a", 10, "commit-a");
        let file_b = TreeFile::new("path-b", "hash-b", 10, "commit-a");

        let file_a_prime = TreeFile::new("path-a", "hash-a-prime", 10, "commit-a");

        let newer = vec![file_a.clone(), file_b.clone()];
        let older = vec![file_a_prime.clone()];

        let result = diff(&newer, &older);

        assert_eq!(result.additions, vec![file_b.clone()]);
        assert_eq!(result.deletions, vec![file_a_prime.clone()]);
        assert_eq!(result.changes, vec![file_a.clone()]);
        Ok(())
    }
}
