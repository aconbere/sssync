use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::migration;
use crate::models::transfer::TransferKind;
use crate::remote::{fetch_remote_db, RemoteInfo};
use crate::s3::make_client;
use crate::store;

pub async fn clone(url_str: &str, destination: &Path) -> Result<()> {
    if destination.exists() {
        return Err(anyhow!(
            "desintation {} already exists",
            destination.display()
        )
        .into());
    }

    let root_path = store::get_root_path(destination);

    if root_path.is_some() {
        return Err(anyhow!(
            "desintation {} is already sssync'd",
            destination.display()
        )
        .into());
    }

    println!("creating: {}", destination.display());
    fs::create_dir(&destination)?;

    let local_path = fs::canonicalize(destination)?;
    let local_db_path = store::db_path(&local_path);
    println!("initializing sssync in: {}", local_path.display());
    store::init(&local_path)?;

    let client = make_client().await;
    let remote_name = "origin";
    let remote_info = RemoteInfo::from_url(url_str)?;

    println!("Fetching remote db");
    let remote_db_path =
        fetch_remote_db(&client, &destination, &remote_name, &remote_info)
            .await?;

    println!(
        "copying database: {} -> {}",
        &remote_db_path.display(),
        &local_db_path.display()
    );
    fs::copy(&remote_db_path, &local_db_path)?;

    println!("opening db: {}", &local_db_path.display());
    let connection = Connection::open(&local_db_path)?;

    println!("fetching meta");
    let meta = db::meta::get(&connection)?;

    println!("fetching head");
    let head = db::commit::get_by_ref_name(&connection, &meta.head)?
        .ok_or(anyhow!("Head is bad - no matching ref name"))?;

    println!("fetching tree");
    let files = db::tree::get(&connection, &head.hash)?;

    let object_hashes: Vec<String> =
        files.iter().map(|f| f.file_hash.clone()).collect();

    println!("creating migration");
    let migration = migration::create(
        &connection,
        TransferKind::Download,
        remote_name,
        &object_hashes,
    )?;
    println!("Starting migration: {}", migration.id);

    migration::run(&connection, &local_path, &migration, false, true).await?;

    println!("Complete");

    for f in files {
        let p = &local_path.join(f.path);
        println!("Copying {} -> {}", f.file_hash, p.display());
        store::export_to(&local_path, &f.file_hash, p)?;
    }

    Ok(())
}
