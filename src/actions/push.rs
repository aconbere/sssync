use std::error::Error;

use rusqlite::Connection;

use crate::db;

pub fn push(connection: &Connection, remote: &str) -> Result<(), Box<dyn Error>> {
    let maybe_head = db::reference::get_head(connection)?;
    let remote = db::remote::get(connection, remote)?;

    println!("Pushing to remote: {} {}", remote.name, remote.url);

    match maybe_head {
        Some(head) => {
            let tree = db::tree::get_tree(connection, &head.hash)?;
            for file in tree {
                println!("file: {}, {}", file.path, file.file_hash);
            }
            Ok(())
        }
        None => Err("no head commit".into()),
    }
}
