use std::error::Error;

use rusqlite::Connection;

use crate::db;

pub fn tree(connection: &Connection, hash: &str) -> Result<(), Box<dyn Error>> {
    match db::tree::get_tree(connection, hash) {
        Ok(tree) => tree.iter().for_each(|t| {
            println!("{}: {}", t.path, t.file_hash);
        }),
        Err(_) => {
            println!("Uknown hash: {}", hash);
        }
    }
    Ok(())
}
