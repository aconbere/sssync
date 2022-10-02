use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db::staging::delete;

pub fn reset(
    connection: &Connection,
    _path: &Path,
) -> Result<(), Box<dyn Error>> {
    delete(connection)?;
    Ok(())
}
