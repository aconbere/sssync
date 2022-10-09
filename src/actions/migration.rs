use std::error::Error;

use rusqlite::Connection;

use crate::db;
use crate::models::migration;
use crate::models::upload;

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let migrations = db::migration::get_all(connection)?;
    migration::print_table(migrations);
    Ok(())
}

pub fn show(
    connection: &Connection,
    migration_id: &str,
) -> Result<(), Box<dyn Error>> {
    let uploads = db::upload::get_all(connection, migration_id)?;
    upload::print_table(uploads);
    Ok(())
}
