use rusqlite::Connection;
use std::error::Error;
use std::path::{Path, PathBuf};

use crate::models::meta::Meta;
use crate::store;

pub mod commit;
pub mod meta;
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

pub fn repo_db_path(root_path: &Path) -> PathBuf {
    let store_path = store::store_path(root_path);
    db_path(&store_path)
}

pub fn init(connection: &Connection) -> Result<(), Box<dyn Error>> {
    commit::create_table(connection)?;
    meta::create_table(connection)?;
    migration::create_table(connection)?;
    reference::create_table(connection)?;
    remote::create_table(connection)?;
    staging::create_table(connection)?;
    tree::create_table(connection)?;
    upload::create_table(connection)?;

    meta::update(connection, &Meta::new("main"))?;
    Ok(())
}
