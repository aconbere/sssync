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
    url_str: &str,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    if destination.exists() {
        return Err(format!(
            "desintation {} already exists",
            destination.display()
        )
        .into());
    }

    let root_path = store::get_root_path(destination);

    if root_path.is_some() {
        return Err(format!(
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

    println!("Fetching remote db");
    let remote_db_path = crate::remote::fetch_remote_database(
        &client,
        destination,
        RemoteKind::S3,
        remote_name,
        url_str,
    )
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
        .ok_or("Head is bad - no matching ref name")?;

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

    migration::run(&connection, &local_path, &migration, false).await?;

    println!("Complete");

    for f in files {
        let p = &local_path.join(f.path);
        println!("Copying {} -> {}", f.file_hash, p.display());
        store::export_to(&local_path, &f.file_hash, p)?;
    }

    Ok(())
}
