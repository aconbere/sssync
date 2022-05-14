use std::error::Error;

use rusqlite::Connection;

use crate::db::remote::insert;
use crate::models::remote::Remote;

pub fn add(connection: &Connection, name: &str, url: &str) -> Result<(), Box<dyn Error>> {
    let remote = Remote::new(name, url)?;
    insert(connection, &remote)
}
