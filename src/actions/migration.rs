use anyhow::Result;
use rusqlite::Connection;

use crate::db;
use crate::models::migration;
use crate::models::transfer;

pub fn list(connection: &Connection) -> Result<()> {
    let migrations = db::migration::get_all(connection)?;
    migration::print_table(migrations);
    Ok(())
}

pub fn show(connection: &Connection, migration_id: &str) -> Result<()> {
    let uploads = db::transfer::get_all(connection, migration_id)?;
    transfer::print_table(uploads);
    Ok(())
}
