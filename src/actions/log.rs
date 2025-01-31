use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;

pub fn log(connection: &Connection) -> Result<()> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("No commit"))?;
    let commits = db::commit::get_children(connection, &head.hash)?;

    commits.into_iter().for_each(|commit| {
        println!("commit {}", commit.hash);
        println!("Author: {}", commit.author);
        println!("Date: {}", commit.created_unix_timestamp);
        println!(
            "Parent: {}",
            commit.parent_hash.unwrap_or_else(|| String::from("None"))
        );
        println!("\n\t{}", commit.comment);
    });
    Ok(())
}
