use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db;
use crate::models::migration::{Migration, MigrationState};
use crate::models::transfer::{Transfer, TransferKind, TransferState};
use crate::remote::RemoteInfo;
use crate::s3::upload_multipart::upload_multipart;
use crate::s3::{download_object, make_client};
use crate::store;

pub fn create(
    connection: &Connection,
    kind: TransferKind,
    remote_name: &str,
    object_hashes: &[String],
) -> Result<Migration> {
    let remote = db::remote::get(connection, remote_name)?;

    let migration = Migration::new(kind.clone(), &remote);
    db::migration::insert(connection, &migration)?;

    let transfers: Vec<Transfer> = object_hashes
        .iter()
        .map(|h| Transfer::new(&migration.id, h, kind.clone()))
        .collect();

    for t in &transfers {
        db::transfer::insert(connection, t)?;
    }

    Ok(migration)
}

pub async fn run(
    connection: &Connection,
    root_path: &Path,
    migration: &Migration,
    force: bool,
    ignore_existing: bool,
) -> Result<()> {
    match migration.kind {
        TransferKind::Upload => {
            run_upload(connection, root_path, migration, force, ignore_existing)
                .await
        }
        TransferKind::Download => {
            run_download(
                connection,
                root_path,
                migration,
                force,
                ignore_existing,
            )
            .await
        }
    }
}

async fn run_upload(
    connection: &Connection,
    root_path: &Path,
    migration: &Migration,
    force: bool,
    ignore_existing: bool,
) -> Result<()> {
    let uploads =
        db::transfer::get_waiting_for_migration(connection, &migration.id)?;

    let client = make_client().await;
    let remote_info = RemoteInfo::from_url(&migration.remote_location)?;
    let upload_count = uploads.len();

    println!("uploading {} files", upload_count);
    db::migration::set_state(connection, migration, MigrationState::Running)?;
    for (i, upload) in uploads.iter().enumerate() {
        let key = remote_info.object_key(&upload.object_hash);

        let local_object_path =
            store::object_path(root_path, &upload.object_hash);

        db::transfer::set_state(connection, upload, TransferState::Running)?;

        println!("Upload {}/{}", i, upload_count);
        println!("\tUploading {} to {}", local_object_path.display(), key,);

        let result = upload_multipart(
            &client,
            &remote_info.bucket,
            &key,
            &local_object_path,
            force,
        )
        .await;

        if result.is_ok() || ignore_existing {
            db::transfer::set_state(
                connection,
                upload,
                TransferState::Complete,
            )?;
        } else {
            db::transfer::set_state(connection, upload, TransferState::Failed)?;
            return Err(result.unwrap_err());
        }
    }
    db::migration::set_state(connection, migration, MigrationState::Complete)?;
    Ok(())
}

pub async fn run_download(
    connection: &Connection,
    root_path: &Path,
    migration: &Migration,
    force: bool,
    ignore_existing: bool,
) -> Result<()> {
    let downloads =
        db::transfer::get_waiting_for_migration(connection, &migration.id)?;
    let download_count = downloads.len();

    let client = make_client().await;

    let remote_info = RemoteInfo::from_url(&migration.remote_location)?;

    println!("Downloading {} files", download_count);
    db::migration::set_state(connection, migration, MigrationState::Running)?;
    for (i, download) in downloads.iter().enumerate() {
        let key = remote_info.object_key(&download.object_hash);

        let local_file_path =
            store::object_path(root_path, &download.object_hash);

        println!("Download {}/{}", i, download_count);
        println!("Downloading {} to {}", key, local_file_path.display());

        // If the file is already in our store skip it
        //
        // If we've set force, overwrite the file
        // If We've set ignore_existing continue
        if store::exists(root_path, &download.object_hash) {
            if ignore_existing {
                continue;
            }
            if !force {
                return Err(anyhow!("File already found: {}, set `force` to override or ignore_existing to ignore", &download.object_hash));
            }
        }

        let mut output_file = File::create(&local_file_path)?;
        let result = download_object(
            &client,
            &remote_info.bucket,
            &key,
            &mut output_file,
        )
        .await;

        match result {
            Ok(_) => {
                db::transfer::set_state(
                    connection,
                    download,
                    TransferState::Complete,
                )?;
                Ok(())
            }
            Err(e) => {
                db::transfer::set_state(
                    connection,
                    download,
                    TransferState::Failed,
                )?;
                Err(e)
            }
        }?
    }
    db::migration::set_state(connection, migration, MigrationState::Complete)?;
    Ok(())
}
