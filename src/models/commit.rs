use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

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
