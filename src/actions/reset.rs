use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use crate::db::staging;

pub fn reset(connection: &Connection, _path: &Path) -> Result<()> {
    staging::delete(connection)?;
    Ok(())
}
