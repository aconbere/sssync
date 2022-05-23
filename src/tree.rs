use crate::models::tree_file::TreeFile;
use std::collections::{HashMap, HashSet};

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
