use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::db::repo_db_path;
use crate::store;

pub fn init(root_path: &Path) -> Result<(), Box<dyn Error>> {
    println!("init: {}", root_path.display());
    store::init(&root_path)?;

    let connection = Connection::open(repo_db_path(&root_path))?;
    println!("found connection: {}", root_path.display());
    db::init(&connection)?;
    Ok(())
}
