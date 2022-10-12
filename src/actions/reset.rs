use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db::staging;

pub fn reset(
    connection: &Connection,
    _path: &Path,
) -> Result<(), Box<dyn Error>> {
    staging::delete(connection)?;
    Ok(())
}
