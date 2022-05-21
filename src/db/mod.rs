use rusqlite::Connection;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::store;

pub mod commit;
pub mod migration;
pub mod reference;
pub mod remote;
pub mod staging;
pub mod tree;
pub mod upload;

pub const DB_FILE_NAME: &str = "sssync.db";

pub fn db_path(path: &Path) -> PathBuf {
    path.join(DB_FILE_NAME)
}

pub fn get_connection(root_path: &Path) -> Result<Connection, Box<dyn Error>> {
    let store_path = store::store_path(root_path);
    let db_path = db_path(&store_path);
    let connection = Connection::open(db_path)?;
    Ok(connection)
}

pub fn init(connection: &Connection) -> Result<(), Box<dyn Error>> {
    staging::create_table(connection)?;
    tree::create_table(connection)?;
    commit::create_table(connection)?;
    reference::create_table(connection)?;
    remote::create_table(connection)?;
    migration::create_table(connection)?;
    upload::create_table(connection)?;
    Ok(())
}
