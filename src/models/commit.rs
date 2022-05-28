use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq)]
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

// Look back through two commit lists and find a shared parent
//
// Assumes that the commits lists are already in parent relationship
// order. So that the vec containing commits a,b,c represents the
// commit list:
//
// a has parent b
// b has parent c
// c has no parent;
//
//
// Example
//
//      let commit_a = Commit::new("a", "", "", None)?;
//      let commit_b = Commit::new("b", "", "", Some(String::from("a")))?;
//      let commit_b = Commit::new("c", "", "", Some(String::from("b")))?;
//
//      let left = vec![commit_c.clone(), commit_b.clone(), commit_a.clone()];
//      let right = vec![commit_b.clone(), commit_a.clone()];
//
//      let result = get_shared_parent(&left, &right);
//
//      assert_eq!(result, Some(commit_b));
//
pub fn get_shared_parent(left: &Vec<Commit>, right: &Vec<Commit>) -> Option<Commit> {
    let mut shared_parent = None;

    let mut left_i = left.len() - 1;
    let mut right_i = right.len() - 1;

    loop {
        match (left.get(left_i), right.get(right_i)) {
            (Some(l), Some(r)) => {
                if l.hash == r.hash {
                    shared_parent = Some(left[left_i].clone());
                    if left_i == 0 || right_i == 0 {
                        break;
                    }
                    left_i -= 1;
                    right_i -= 1;
                } else {
                    break;
                }
            }
            _ => break,
        }
    }

    return shared_parent.clone();
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
        println!("diff_commit_list:shared_parent: {:?}", shared_parent);
        CompareResult::Diff {
            left: commits_since(&left, &shared_parent).unwrap(),
            right: commits_since(&right, &shared_parent).unwrap(),
        }
    } else {
        return CompareResult::NoSharedParent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_shared_parent_with_equal_lists() -> Result<(), Box<dyn Error>> {
        let commit_a = Commit::new("a", "", "", None)?;
        let commit_b = Commit::new("b", "", "", Some(String::from("a")))?;

        let left = vec![commit_b.clone(), commit_a.clone()];
        let right = vec![commit_b.clone(), commit_a.clone()];

        let result = get_shared_parent(&left, &right);
        assert_eq!(result, Some(commit_b));
        Ok(())
    }

    #[test]
    fn test_get_shared_parent_with_left_longer() -> Result<(), Box<dyn Error>> {
        let commit_a = Commit::new("a", "", "", None)?;
        let commit_b = Commit::new("b", "", "", Some(String::from("a")))?;
        let commit_c = Commit::new("c", "", "", Some(String::from("b")))?;

        let left = vec![commit_c.clone(), commit_b.clone(), commit_a.clone()];
        let right = vec![commit_b.clone(), commit_a.clone()];

        let result = get_shared_parent(&left, &right);
        assert_eq!(result, Some(commit_b));
        Ok(())
    }

    #[test]
    fn test_get_shared_parent_with_right_longer() -> Result<(), Box<dyn Error>> {
        let commit_a = Commit::new("a", "", "", None)?;
        let commit_b = Commit::new("b", "", "", Some(String::from("a")))?;
        let commit_c = Commit::new("c", "", "", Some(String::from("b")))?;

        let left = vec![commit_b.clone(), commit_a.clone()];
        let right = vec![commit_c.clone(), commit_b.clone(), commit_a.clone()];

        let result = get_shared_parent(&left, &right);
        assert_eq!(result, Some(commit_b));
        Ok(())
    }

    #[test]
    fn test_diff_commit_list_right_longer() -> Result<(), Box<dyn Error>> {
        let commit_a = Commit::new("a", "", "", None)?;
        let commit_b = Commit::new("b", "", "", Some(String::from("a")))?;

        let left = vec![commit_a.clone()];
        let right = vec![commit_b.clone(), commit_a.clone()];

        match diff_commit_list(&left, &right) {
            CompareResult::Diff {
                left: _,
                right: right_diff,
            } => {
                assert_eq!(right_diff, vec!(commit_b));
                Ok(())
            }
            CompareResult::NoSharedParent => Err("should be a diff".into()),
        }
    }

    #[test]
    fn test_diff_commit_list_left_longer() -> Result<(), Box<dyn Error>> {
        let commit_a = Commit::new("a", "", "", None)?;
        let commit_b = Commit::new("b", "", "", Some(String::from("a")))?;

        let left = vec![commit_b.clone(), commit_a.clone()];
        let right = vec![commit_a.clone()];

        match diff_commit_list(&left, &right) {
            CompareResult::Diff {
                left: left_diff,
                right: _,
            } => {
                assert_eq!(left_diff, vec!(commit_b));
                Ok(())
            }
            CompareResult::NoSharedParent => Err("should be a diff".into()),
        }
    }
}
