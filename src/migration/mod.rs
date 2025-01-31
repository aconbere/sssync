use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::models::migration::{Migration, MigrationState};
use crate::models::transfer::{Transfer, TransferKind, TransferState};
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

fn remote_object_path(remote_path: &Path, hash: &str) -> PathBuf {
    remote_path.join(".sssync/objects").join(hash)
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

    println!("uploading {} files", uploads.len());
    let client = make_client().await;
    let u = Url::parse(&migration.remote_location)?;

    // for a url like `s3://anders.conbere.org/games` the url decomposes to
    // bucket: anders.conbere.org
    // key: /games
    let bucket = u.host_str().unwrap();
    let remote_directory = Path::new(u.path());

    let upload_count = uploads.len();

    db::migration::set_state(connection, migration, MigrationState::Running)?;
    for (i, upload) in uploads.iter().enumerate() {
        let remote_object_path =
            remote_object_path(remote_directory, &upload.object_hash);

        let local_object_path =
            store::object_path(root_path, &upload.object_hash);

        db::transfer::set_state(connection, upload, TransferState::Running)?;
        println!("Upload {}/{}", i, upload_count);
        println!(
            "Uploading {} to {}",
            local_object_path.display(),
            remote_object_path.display()
        );

        let result = upload_multipart(
            &client,
            bucket,
            &remote_object_path,
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

    println!("Downloading {} files", downloads.len());
    let client = make_client().await;
    let u = Url::parse(&migration.remote_location)?;

    // for a url like `s3://anders.conbere.org/games` the url decomposes to
    // bucket: anders.conbere.org
    // key: /games
    let bucket = u.host_str().unwrap();
    let remote_directory = Path::new(u.path());

    let download_count = downloads.len();

    db::migration::set_state(connection, migration, MigrationState::Running)?;
    for (i, download) in downloads.iter().enumerate() {
        let remote_object_path =
            remote_object_path(remote_directory, &download.object_hash);

        let local_object_path =
            store::object_path(root_path, &download.object_hash);

        let mut copy_file = File::create(&local_object_path)?;

        println!("Download {}/{}", i, download_count);
        println!(
            "Downloading {} to {}",
            remote_object_path.display(),
            local_object_path.display(),
        );

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

        let result = download_object(
            &client,
            bucket,
            remote_object_path.to_str().unwrap(),
            &mut copy_file,
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
