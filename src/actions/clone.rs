use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models;
use crate::remote::{fetch_remote_db, fetch_remote_objects, RemoteInfo};
use crate::s3::make_client;
use crate::store;

pub async fn clone(url_str: &str, destination: &Path) -> Result<()> {
    let remote_info = RemoteInfo::from_url(&url_str)?;

    if destination.exists() {
        return Err(anyhow!(
            "desintation {} already exists",
            destination.display()
        )
        .into());
    }

    let Some(root_path) = store::get_root_path(destination) else {
        return Err(anyhow!(
            "desintation {} is already sssync'd",
            destination.display()
        ));
    };

    println!("creating: {}", destination.display());
    fs::create_dir(&destination)?;
    let local_path = fs::canonicalize(destination)?;
    let local_db_path = store::db_path(&local_path);

    println!("initializing sssync in: {}", local_path.display());
    store::init(&local_path)?;
    let remote_name = "origin";
    let connection = Connection::open(&local_db_path)?;
    let client = make_client().await;

    db::remote::insert(
        &connection,
        &models::remote::Remote {
            name: remote_name.to_string(),
            kind: remote_info.kind,
            location: url_str.to_string(),
        },
    )?;

    println!("Fetching remote db");
    fetch_remote_db(&client, &remote_info, &destination).await?;
    fetch_remote_objects(&connection, &root_path, remote_name).await?;

    Ok(())
}
