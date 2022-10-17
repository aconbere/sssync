use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq, Eq)]
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
            parent_hash,
            hash: hash.to_string(),
            comment: comment.to_string(),
            author: author.to_string(),
            created_unix_timestamp: time.as_secs(),
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
pub fn get_shared_parent(left: &[Commit], right: &[Commit]) -> Option<Commit> {
    let mut shared_parent = None;

    let mut left_i = left.len() - 1;
    let mut right_i = right.len() - 1;

    while let (Some(l), Some(r)) = (left.get(left_i), right.get(right_i)) {
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

    shared_parent
}

/* Takes a list of ordered commits and returns the tail of that list following a commit who's hash
 * matches "needle".
 */
pub fn commits_since(haystack: &Vec<Commit>, needle: &Commit) -> Vec<Commit> {
    let mut diff: Vec<Commit> = vec![];

    for l in haystack {
        if l.hash == needle.hash {
            break;
        }
        diff.push(l.clone());
    }

    diff
}

pub enum CompareResult {
    Diff {
        left: Vec<Commit>,
        right: Vec<Commit>,
    },
    NoSharedParent,
}

/* Takes two ordered set of commits, finds their shared parent commit,
 * and returns each of their remainders since that parent commit
 */
pub fn diff_commit_list(
    left: &Vec<Commit>,
    right: &Vec<Commit>,
) -> CompareResult {
    if let Some(shared_parent) = get_shared_parent(left, right) {
        CompareResult::Diff {
            left: commits_since(left, &shared_parent),
            right: commits_since(right, &shared_parent),
        }
    } else {
        CompareResult::NoSharedParent
    }
}

// Return the set of commits in left that aren't found in right
pub fn diff_commit_list_left(
    left: &Vec<Commit>,
    right: &Vec<Commit>,
) -> Result<Vec<Commit>, Box<dyn Error>> {
    println!("Left: {:?}", left);
    println!("Right: {:?}", right);
    match diff_commit_list(left, right) {
        CompareResult::NoSharedParent => {
            Err("Remote has no shared parent".into())
        }
        CompareResult::Diff { left, right } => {
            if !right.is_empty() {
                Err("no fast forward, remote has commits not in the current db"
                    .into())
            } else if left.is_empty() {
                Err("no differences between remote and local".into())
            } else {
                Ok(left)
            }
        }
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
    fn test_get_shared_parent_with_right_longer() -> Result<(), Box<dyn Error>>
    {
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
