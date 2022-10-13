use std::error::Error;
use std::fs;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::migration;
use crate::models::transfer::TransferKind;
use crate::s3::make_client;
use crate::store;
use crate::types::remote_kind::RemoteKind;

pub async fn clone(
    url_str: &String,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    if !destination.is_dir() {
        return Err(format!(
            "desintation {} must be a directory",
            destination.display()
        )
        .into());
    }

    let root_path = store::get_root_path(&destination);

    if root_path.is_some() {
        return Err(format!(
            "desintation {} is already sssync'd",
            destination.display()
        )
        .into());
    }

    println!("initializing sssync in: {}", destination.display());
    store::init(&destination)?;

    let client = make_client().await;

    let remote_name = "origin";

    let remote_path = crate::remote::fetch_remote_database(
        &client,
        &destination,
        RemoteKind::S3,
        remote_name,
        url_str,
    )
    .await?;

    let local_path = fs::canonicalize(destination)?;
    fs::copy(&remote_path, &local_path)?;

    let connection = Connection::open(&remote_path)?;

    let meta = db::meta::get(&connection)?;
    let head = db::commit::get_by_ref_name(&connection, &meta.head)?
        .ok_or("Head is bad - no matching ref name")?;

    let files = db::tree::get(&connection, &head.hash)?;

    let object_hashes: Vec<String> =
        files.into_iter().map(|f| f.file_hash).collect();

    migration::create(
        &connection,
        TransferKind::Download,
        remote_name,
        &object_hashes,
    )?;

    Ok(())
}
