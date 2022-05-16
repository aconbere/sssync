use std::error::Error;

use rusqlite::Connection;

use crate::db;

pub fn log(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let head = db::reference::get_head(connection)?;

    match head {
        Some(head_commit) => {
            let commits = db::commit::get_all(connection, &head_commit.hash)?;

            commits.into_iter().for_each(|commit| {
                println!("commit {}", commit.hash);
                println!("Author: {}", commit.author);
                println!("Date: {}", commit.created_unix_timestamp);
                println!(
                    "Parent: {}",
                    commit.parent_hash.unwrap_or(String::from("None"))
                );
                println!("");
                println!("\t{}", commit.comment);
            });
            Ok(())
        }
        None => {
            println!("no commits made yet");
            Ok(())
        }
    }
}
