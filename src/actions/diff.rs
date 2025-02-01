use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::db::tree::diff_commits;

pub fn diff(connection: &Connection, hash: &str) -> Result<()> {
    let staged_files = db::staging::get_all(connection)?;

    if !staged_files.is_empty() {
        return Err(anyhow!("There are currently staged files: Commit your current work or reset your state to continue"));
    }

    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("Head is bad - no matching ref name"))?;

    let diff = diff_commits(&connection, &head.hash, hash)?;

    for f in diff.additions {
        println!("Added: {}", f.path)
    }

    for f in diff.changes {
        println!("changed: {}", f.path)
    }

    for f in diff.deletions {
        println!("Removed: {}", f.path)
    }

    Ok(())
}
