use std::error::Error;

use rusqlite::Connection;

use crate::db;
use crate::models;

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

pub fn switch(connection: &Connection, name: &str) -> Result<(), Box<dyn Error>> {
    let commit =
        db::commit::get_by_ref_name(connection, name)?.ok_or("Could not find hash for ref")?;
    let tree = db::tree::get(connection, &commit.hash)?;
    println!("Switch branches: {}", tree.len());
    Ok(())
}

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let branches = db::reference::get_all_by_kind(connection, models::reference::Kind::Branch)?;

    println!("Branches:");
    for b in branches {
        println!("\t{}", b.name)
    }
    Ok(())
}
