use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models;
use crate::models::meta::Meta;
use crate::tree::TreeDiff;

pub fn add(
    connection: &Connection,
    name: &str,
    hash: Option<String>,
) -> Result<()> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?;

    let commit_hash = match (hash, head) {
        (Some(_hash), None) => _hash,
        (None, Some(_head)) => _head.hash,
        _ => return Err(anyhow!("Could not find hash")),
    };

    db::reference::insert(
        connection,
        name,
        models::reference::Kind::Branch,
        &commit_hash,
        None,
    )?;
    Ok(())
}

pub fn switch(
    connection: &Connection,
    root_path: &Path,
    name: &str,
) -> Result<()> {
    let staged_files = db::staging::get_all(connection)?;

    if !staged_files.is_empty() {
        return Err(anyhow!("There are currently staged files: Commit your current work or reset your state to continue"));
    }

    let reference = db::reference::get(connection, None, name)?;
    let commit = db::commit::get(connection, &reference.hash)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("Head is bad - no matching ref name"))?;

    let current_tree = db::tree::get(connection, &head.hash)?;
    let future_tree = db::tree::get(connection, &commit.hash)?;

    let diff = TreeDiff::new(&current_tree, &future_tree);
    diff.apply(root_path)?;
    db::meta::update(connection, &Meta::new(&reference.name))
}

pub fn list(connection: &Connection) -> Result<()> {
    let branches = db::reference::get_all_by_kind(
        connection,
        None,
        models::reference::Kind::Branch,
    )?;

    println!("Branches:");
    for b in branches {
        println!("\t{}", b.name)
    }
    Ok(())
}
