use std::collections::{HashMap, HashSet};

use crate::models::tree_file::TreeFile;

#[derive(Debug, PartialEq)]
pub struct DiffResult {
    pub additions: Vec<TreeFile>,
    pub deletions: Vec<TreeFile>,
    pub changes: Vec<TreeFile>,
}

// The Problem:
//
// On the left and right we have some file system state, both files, and their hashes.
//
// We want to know:
//
// a: what files were added?
// b: what files were deleted?
// c: what files were changed?
//
// to do this we're going to first focus on file_hashes.
//
// We'll build a set of file_hashes for the left and right.
//
// Any file_hashes in left but not in right are either New or Changed.
// Any file_hashes in right but not in left are deletions.
//
// Now that we have new or chagned, we'll walk through the new or changed
// files and look up if the path exists in the right.
//
// If the file is new or changed, but the path exists in right then the file
// is changed, otherwise it's new.
pub fn diff_trees(left: &Vec<TreeFile>, right: &Vec<TreeFile>) -> DiffResult {
    println!("diff_trees left: {:?}", left);
    println!("diff_trees right: {:?}", right);
    let mut left_set: HashMap<String, TreeFile> = HashMap::new();

    for tree_file in left {
        left_set.insert(tree_file.file_hash.clone(), tree_file.clone());
    }

    let mut right_set: HashMap<String, TreeFile> = HashMap::new();
    let mut right_paths: HashSet<String> = HashSet::new();

    for tree_file in right {
        right_set.insert(tree_file.file_hash.clone(), tree_file.clone());
        right_paths.insert(tree_file.path.clone());
    }

    let mut new_or_changed: Vec<TreeFile> = vec![];

    for tree_file in left_set.values() {
        if !right_set.contains_key(&tree_file.file_hash) {
            new_or_changed.push(tree_file.clone());
        }
    }

    let mut changes = vec![];
    let mut additions = vec![];

    for tree_file in new_or_changed {
        if right_paths.contains(&tree_file.path) {
            changes.push(tree_file.clone());
        } else {
            additions.push(tree_file.clone());
        }
    }

    let mut deletions: Vec<TreeFile> = vec![];
    for tree_file in right_set.values() {
        if !left_set.contains_key(&tree_file.file_hash) {
            deletions.push(tree_file.clone());
        }
    }

    DiffResult {
        additions: additions,
        deletions: deletions,
        changes: changes,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_diff_trees_empty_trees() -> Result<(), Box<dyn Error>> {
        let left = vec![];
        let right = vec![];

        let result = diff_trees(&left, &right);
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
    fn test_diff_trees_left_simple_addition() -> Result<(), Box<dyn Error>> {
        let file_a = TreeFile::new("path-a", "hash-a", 10, "commit-a");
        let file_b = TreeFile::new("path-b", "hash-b", 10, "commit-a");

        let left = vec![file_a.clone(), file_b.clone()];
        let right = vec![];

        let result = diff_trees(&left, &right);
        assert!(result.deletions.is_empty());
        assert!(result.changes.is_empty());
        assert!(result.additions.contains(&file_a));
        assert!(result.additions.contains(&file_b));
        Ok(())
    }

    #[test]
    fn test_diff_trees_change_in_a() -> Result<(), Box<dyn Error>> {
        let file_a = TreeFile::new("path-a", "hash-a", 10, "commit-a");
        let file_b = TreeFile::new("path-b", "hash-b", 10, "commit-a");

        let file_a_prime = TreeFile::new("path-a", "hash-a-prime", 10, "commit-a");

        let left = vec![file_a.clone(), file_b.clone()];
        let right = vec![file_a_prime.clone()];

        let result = diff_trees(&left, &right);

        assert_eq!(result.additions, vec![file_b.clone()]);
        assert_eq!(result.deletions, vec![file_a_prime.clone()]);
        assert_eq!(result.changes, vec![file_a.clone()]);
        Ok(())
    }
}
