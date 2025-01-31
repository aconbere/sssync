use anyhow::Result;
use rusqlite::Connection;

use crate::db;

pub fn tree(connection: &Connection, hash: &str) -> Result<()> {
    match db::tree::get(connection, hash) {
        Ok(tree) => tree.iter().for_each(|t| {
            println!("{}: {}", t.path, t.file_hash);
        }),
        Err(_) => {
            println!("Uknown hash: {}", hash);
        }
    }
    Ok(())
}
