use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models;
use crate::models::meta::Meta;
use crate::store;
use crate::tree::TreeDiff;

pub fn show(connection: &Connection) -> Result<()> {
    let meta = db::meta::get(connection)?;

    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("Head is bad - no matching ref name"))?;

    println!("On branch: {}", meta.head);
    println!("\tref: {}", head.hash);

    Ok(())
}

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

    let reference = db::reference::get(connection, name)?;
    let commit = db::commit::get(connection, &reference.hash)?;
    let meta = db::meta::get(connection)?;

    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("Head is bad - no matching ref name"))?;

    let current_tree = db::tree::get(connection, &head.hash)?;
    let future_tree = db::tree::get(connection, &commit.hash)?;

    let diff = TreeDiff::new(&current_tree, &future_tree);

    store::apply_diff(root_path, &diff)?;
    db::meta::update(connection, &Meta::new(&reference.name))
}

/* Lists all branches in the local repository
 */
pub fn list(connection: &Connection) -> Result<()> {
    let meta = db::meta::get(connection)?;

    let branches = db::reference::get_all_by_kind(
        connection,
        models::reference::Kind::Branch,
    )?;

    println!("Branches:");
    for b in branches {
        if meta.head == b.name {
            println!("\t* {}", b.name)
        } else {
            println!("\t{}", b.name)
        }
    }
    Ok(())
}
