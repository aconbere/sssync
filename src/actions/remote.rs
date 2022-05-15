use std::error::Error;

use rusqlite::Connection;

use crate::db::remote::{get_all, insert};
use crate::models::remote::Remote;

pub fn add(connection: &Connection, name: &str, url: &str) -> Result<(), Box<dyn Error>> {
    let remote = Remote::new(name, url)?;
    insert(connection, &remote)
}

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let remotes = get_all(connection)?;

    for remote in remotes {
        println!("Remote: {} {}", remote.name, remote.url);
    }

    Ok(())
}
