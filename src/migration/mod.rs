use std::error::Error;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::models::migration::{Migration, MigrationKind, MigrationState};
use crate::models::upload::{Upload, UploadState};
use crate::s3::make_client;
use crate::s3::upload_multipart::upload_multipart;

pub fn create(
    connection: &Connection,
    kind: MigrationKind,
    remote_name: &str,
    object_hashes: &Vec<String>,
) -> Result<Migration, Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let migration = Migration::new(kind, &remote);
    db::migration::insert(connection, &migration)?;

    let uploads: Vec<Upload> = object_hashes
        .iter()
        .map(|h| Upload::new(&migration.id, &h))
        .collect();

    for upload in &uploads {
        db::upload::insert(connection, upload)?;
    }

    Ok(migration)
}

pub async fn run(
    connection: &Connection,
    root_path: &Path,
    migration: &Migration,
) -> Result<(), Box<dyn Error>> {
    let uploads = db::upload::get_waiting_for_migration(connection, migration)?;

    let client = make_client().await;
    let u = Url::parse(&migration.remote_location)?;
    let bucket = u.host_str().unwrap();
    let remote_directory = Path::new(u.path()).join(&migration.remote_name);

    db::migration::set_state(connection, &migration, MigrationState::Running)?;
    for upload in uploads {
        let remote_object_path = remote_directory
            .join(".sssync/objects")
            .join(&upload.object_hash);

        let local_object_path = root_path.join(".sssync/objects").join(&upload.object_hash);

        db::upload::set_state(connection, &upload, UploadState::Running)?;
        println!(
            "Uploading {} to {}",
            local_object_path.display(),
            remote_object_path.display()
        );
        match upload_multipart(&client, bucket, &local_object_path, &remote_object_path).await {
            Ok(_) => {
                db::upload::set_state(connection, &upload, UploadState::Complete)?;
                Ok(())
            }
            Err(e) => {
                db::upload::set_state(connection, &upload, UploadState::Failed)?;
                Err(e)
            }
        }?
    }
    db::migration::set_state(connection, &migration, MigrationState::Complete)?;

    Ok(())
}
