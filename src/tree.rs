use crate::models::tree_file::TreeFile;
use std::collections::HashSet;

struct DiffResult {
    additions: HashSet<TreeFile>,
    deletions: HashSet<TreeFile>,
}

pub fn diff(left: &Vec<TreeFile>, right: &Vec<TreeFile>) -> DiffResult {
    DiffResult {
        additions: vec![],
        deletions: vec![],
    }
}
