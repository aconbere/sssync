use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::store;

pub fn log(
    connection: &Connection,
    root_path: &Path,
    maybe_hash: Option<String>,
    maybe_branch_name: Option<String>,
    maybe_remote_name: Option<String>,
) -> Result<()> {
    let meta = db::meta::get(connection)?;

    // If we are pointing at a remote we should make requests
    // to the remote database
    let target_connection = if let Some(remote_name) = maybe_remote_name {
        let remote_db_path = store::remote_db_path(root_path, &remote_name)?;
        println!("Connection to: {}", remote_db_path.display());
        &Connection::open(&remote_db_path)?
    } else {
        connection
    };

    let starting_hash = if let Some(hash) = maybe_hash {
        if maybe_branch_name.is_some() {
            return Err(anyhow!(
                "can't specify a hash while also including branch"
            ));
        }
        hash
    } else if let Some(branch_name) = maybe_branch_name {
        println!("Looking for : {}", branch_name);
        let reference = db::reference::get(target_connection, &branch_name)?;
        reference.hash
    } else {
        let head = db::commit::get_by_ref_name(target_connection, &meta.head)?
            .ok_or(anyhow!("Invalid head, no commit found"))?;
        head.hash
    };
    println!("Showing commits from: {}", starting_hash);

    let commits = db::commit::get_children(target_connection, &starting_hash)?;

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
