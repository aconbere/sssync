use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::db::repo_db_path;
use crate::store;

pub fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.is_dir() {
        return Err(format!(
            "desintation {} must be a directory",
            path.display()
        )
        .into());
    }

    let root_path = store::get_root_path(path);

    if root_path.is_some() {
        return Err(format!(
            "desintation {} is already sssync'd",
            path.display()
        )
        .into());
    }
    println!("initializing sssync in: {}", path.display());
    store::init(path)?;

    let connection = Connection::open(repo_db_path(path))?;
    db::init(&connection)?;
    Ok(())
}
