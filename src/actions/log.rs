use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db::commit::get_all;

pub fn log(connection: &Connection, _path: &Path) -> Result<(), Box<dyn Error>> {
    let commits = get_all(connection)?;
    commits.into_iter().for_each(|commit| {
        println!("commit {}", commit.hash);
        println!("Author: {}", commit.author);
        println!("Date: {}", commit.created_unix_timestamp);
        println!("");
        println!("\t{}", commit.comment);
    });
    Ok(())
}
