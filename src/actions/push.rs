use std::error::Error;

use rusqlite::Connection;

use crate::db;
use crate::remote;

pub fn push(connection: &Connection, remote: &str) -> Result<(), Box<dyn Error>> {
    let maybe_head = db::reference::get_head(connection)?;
    let head = maybe_head.ok_or(String::from("no head"))?;

    let remote = db::remote::get(connection, remote)?;

    println!(
        "Pushing {} to remote: {} {} {}",
        head.hash, remote.name, remote.kind, remote.location
    );

    remote::download(&remote)?;

    // download the remote db

    Ok(())
}
