use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};

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
    object_hashes: &Vec<String>,
) -> Result<Migration, Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;

    let migration = Migration::new(kind.clone(), &remote);
    db::migration::insert(connection, &migration)?;

    let transfers: Vec<Transfer> = object_hashes
        .iter()
        .map(|h| Transfer::new(&migration.id, &h, kind.clone()))
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
) -> Result<(), Box<dyn Error>> {
    match migration.kind {
        TransferKind::Upload => {
            run_upload(connection, root_path, migration, force).await
        }
        TransferKind::Download => {
            run_download(connection, root_path, migration, force).await
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
) -> Result<(), Box<dyn Error>> {
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

    db::migration::set_state(connection, &migration, MigrationState::Running)?;
    for (i, upload) in uploads.iter().enumerate() {
        let remote_object_path =
            remote_object_path(remote_directory, &upload.object_hash);

        let local_object_path =
            store::object_path(root_path, &upload.object_hash);

        db::transfer::set_state(connection, &upload, TransferState::Running)?;
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

        match result {
            Ok(_) => {
                db::transfer::set_state(
                    connection,
                    &upload,
                    TransferState::Complete,
                )?;
                Ok(())
            }
            Err(e) => {
                db::transfer::set_state(
                    connection,
                    &upload,
                    TransferState::Failed,
                )?;
                Err(e)
            }
        }?
    }
    db::migration::set_state(connection, &migration, MigrationState::Complete)?;
    Ok(())
}

pub async fn run_download(
    connection: &Connection,
    root_path: &Path,
    migration: &Migration,
    force: bool,
) -> Result<(), Box<dyn Error>> {
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

    db::migration::set_state(connection, &migration, MigrationState::Running)?;
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
        if store::exists(root_path, &download.object_hash) && !force {
            continue;
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
                    &download,
                    TransferState::Complete,
                )?;
                Ok(())
            }
            Err(e) => {
                db::transfer::set_state(
                    connection,
                    &download,
                    TransferState::Failed,
                )?;
                Err(e)
            }
        }?
    }
    db::migration::set_state(connection, &migration, MigrationState::Complete)?;
    Ok(())
}
