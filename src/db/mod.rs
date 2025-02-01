use rusqlite::Connection;
use std::path::{Path, PathBuf};

use crate::models;
use crate::store;
use anyhow::Result;

pub mod commit;
pub mod meta;
pub mod migration;
pub mod reference;
pub mod remote;
pub mod staging;
pub mod transfer;
pub mod tree;

pub const DB_FILE_NAME: &str = "sssync.db";

pub fn db_path(path: &Path) -> PathBuf {
    path.join(DB_FILE_NAME)
}

pub fn repo_db_path(root_path: &Path) -> PathBuf {
    let store_path = store::store_path(root_path);
    db_path(&store_path)
}

pub fn init(connection: &Connection) -> Result<()> {
    commit::create_table(connection)?;
    meta::create_table(connection)?;
    migration::create_table(connection)?;
    reference::create_table(connection)?;
    remote::create_table(connection)?;
    staging::create_table(connection)?;
    tree::create_table(connection)?;
    transfer::create_table(connection)?;

    meta::update(connection, &models::meta::Meta::new("main"))?;
    Ok(())
}

pub fn update_remote(
    local_connection: &Connection,
    remote_connection: &Connection,
) -> Result<()> {
    let local_commits = commit::get_all(local_connection)?;
    for c in local_commits {
        commit::insert(remote_connection, &c)?;
    }

    let local_trees = tree::get_all(local_connection)?;
    for t in local_trees {
        tree::insert(remote_connection, &t)?;
    }

    let remote_meta = meta::get(remote_connection)?;
    let local_meta = meta::get(local_connection)?;

    reference::update(
        &remote_connection,
        &remote_meta.head,
        models::reference::Kind::Branch,
        &local_meta.head,
        None,
    )?;

    Ok(())
}
