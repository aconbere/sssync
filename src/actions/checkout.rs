use std::error::Error;

use rusqlite::Connection;

use crate::db::commit;
use crate::db::tree;

pub fn checkout(connection: &Connection, hash: &str) -> Result<(), Box<dyn Error>> {
    let commit = commit::get(connection, hash)?;
    println!("checkout commit:");
    let tree = tree::get_tree(connection, &commit.hash)?;
    println!("checkout tree: {}", tree.len());

    tree.into_iter().for_each(|tree_entry| {
        println!("file: {}:{}", tree_entry.path, tree_entry.file_hash);
    });

    Ok(())
}
