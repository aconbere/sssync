use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::models;
use crate::models::meta::Meta;
use crate::tree::TreeDiff;

pub fn add(
    connection: &Connection,
    name: &str,
    hash: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?;

    let commit_hash = if hash.is_some() {
        hash.unwrap()
    } else if head.is_some() {
        head.unwrap().hash
    } else {
        return Err("Could not find hash".into());
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
) -> Result<(), Box<dyn Error>> {
    let staged_files = db::staging::get_all(connection)?;

    if !staged_files.is_empty() {
        return Err("There are currently staged files: Commit your current work or reset your state to continue".into());
    }

    let reference = db::reference::get(connection, name)?;
    let commit = db::commit::get(connection, &reference.hash)?;
    let future_tree = db::tree::get(connection, &commit.hash)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or("Head is bad - no matching ref name")?;
    let current_tree = db::tree::get(connection, &head.hash)?;

    let diff = TreeDiff::new(&current_tree, &future_tree);
    diff.apply(root_path)?;
    db::meta::update(connection, &Meta::new(&reference.name))
}

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let branches = db::reference::get_all_by_kind(
        connection,
        models::reference::Kind::Branch,
    )?;

    println!("Branches:");
    for b in branches {
        println!("\t{}", b.name)
    }
    Ok(())
}
