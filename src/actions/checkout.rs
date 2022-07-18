use std::error::Error;

use rusqlite::Connection;

use crate::db;
use crate::tree;

pub fn checkout(connection: &Connection, hash: &str) -> Result<(), Box<dyn Error>> {
    let staged_files = db::staging::get_all(connection)?;

    if !staged_files.is_empty() {
        return Err("There are currently staged files: Commit your current work or reset your state to continue".into());
    }

    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?;

    match head {
        Some(head) => checkout_diff(connection, &head.hash, hash),
        None => checkout_fresh(connection, hash),
    }
}

fn checkout_fresh(connection: &Connection, hash: &str) -> Result<(), Box<dyn Error>> {
    let commit = db::commit::get(connection, hash)?;
    println!("checkout commit:");
    let tree = db::tree::get(connection, &commit.hash)?;
    println!("checkout tree: {}", tree.len());

    tree.into_iter().for_each(|tree_entry| {
        println!("file: {}:{}", tree_entry.path, tree_entry.file_hash);
    });

    Ok(())
}

fn checkout_diff(
    connection: &Connection,
    current_hash: &str,
    new_hash: &str,
) -> Result<(), Box<dyn Error>> {
    let current_tree = db::tree::get(connection, current_hash)?;
    let new_tree = db::tree::get(connection, new_hash)?;
    let diff = tree::diff(&current_tree, &new_tree);

    for f in diff.additions {}

    Ok(())
}
