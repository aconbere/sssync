use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Commit {
    pub hash: String,
    pub comment: String,
    pub author: String,
    pub created_unix_timestamp: u64,
    pub parent_hash: Option<String>,
}

impl Commit {
    pub fn new(
        hash: &str,
        comment: &str,
        author: &str,
        parent_hash: Option<String>,
    ) -> Result<Commit, Box<dyn Error>> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?;
        Ok(Commit {
            hash: hash.to_string(),
            comment: comment.to_string(),
            author: author.to_string(),
            created_unix_timestamp: time.as_secs(),
            parent_hash: parent_hash,
        })
    }
}

pub fn get_shared_parent(left: &Vec<Commit>, right: &Vec<Commit>) -> Option<Commit> {
    let left_last = match left.last() {
        Some(l) => l,
        None => {
            return None;
        }
    };

    let right_last = match right.last() {
        Some(r) => r,
        None => {
            return None;
        }
    };

    if left_last.hash != right_last.hash {
        return None;
    }

    let mut shared_parent = left_last;

    for i in left.len()..1 {
        match (left.get(i), right.get(i)) {
            (Some(l), Some(r)) => {
                if l.hash == r.hash {
                    shared_parent = &left[i];
                    break;
                }
            }
            _ => break,
        }
    }

    return Some(shared_parent.clone());
}

pub fn commits_since(list: &Vec<Commit>, parent: &Commit) -> Option<Vec<Commit>> {
    let mut diff: Vec<Commit> = vec![];
    let mut found = false;

    for i in 0..list.len() {
        let l = &list[i];

        if l.hash == parent.hash {
            found = true;
            break;
        }
        diff.push(list[i].clone());
    }

    if !found {
        None
    } else {
        Some(diff)
    }
}

pub enum CompareResult {
    Diff {
        left: Vec<Commit>,
        right: Vec<Commit>,
    },
    NoSharedParent,
}

pub fn diff_commit_list(left: &Vec<Commit>, right: &Vec<Commit>) -> CompareResult {
    if let Some(shared_parent) = get_shared_parent(left, right) {
        CompareResult::Diff {
            left: commits_since(&left, &shared_parent).unwrap(),
            right: commits_since(&right, &shared_parent).unwrap(),
        }
    } else {
        return CompareResult::NoSharedParent;
    }
}
