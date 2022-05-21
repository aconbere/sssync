use std::error::Error;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::models::commit;
use crate::models::migration::MigrationKind;
use crate::models::remote::Remote;
use crate::s3::{make_client, upload_object};
use crate::types::remote_kind::RemoteKind;

pub fn add(
    connection: &Connection,
    name: &str,
    kind: &RemoteKind,
    location: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = Remote::new(name, kind.clone(), location)?;
    db::remote::insert(connection, &remote)
}

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let remotes = db::remote::get_all(connection)?;

    for remote in remotes {
        println!("Remote: {} {}", remote.name, remote.location);
    }

    Ok(())
}

pub async fn init(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let head = db::reference::get_head(connection)?;

    if head.is_none() {
        return Err("no valid head".into());
    }

    let head = head.unwrap();

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;
            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);
            let remote_db_path = remote_directory.join(".sssync/sssync.db");
            let local_db_path = root_path.join(".sssync/sssync.db");
            println!("Uploading database");
            upload_object(&client, bucket, &local_db_path, &remote_db_path).await?;

            let tree = db::tree::get_tree(connection, &head.hash)?;
            let hashes = tree.iter().map(|t| t.file_hash.to_string()).collect();
            println!("Saving migration");
            let migration =
                crate::migration::create(connection, MigrationKind::Upload, &remote.name, &hashes)?;

            println!("Running Migration");
            crate::migration::run(connection, root_path, &migration).await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn push(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let head = db::reference::get_head(connection)?.ok_or("no valid head")?;
    let commits = db::commit::get_all(connection, &head.hash)?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);

            let remote_db_copy =
                crate::remote::fetch_remote_database(&client, &remote, &root_path).await?;
            let remote_db_connection = db::get_connection(&remote_db_copy)?;
            let remote_head = db::reference::get_head(&remote_db_connection)?
                .ok_or("Remote has no valid head")?;
            let remote_commits = db::commit::get_all(&remote_db_connection, &remote_head.hash)?;

            match commit::diff(&commits, &remote_commits) {
                commit::CompareResult::NoSharedParent => {
                    return Err("Remote has no shared parent".into())
                }
                commit::CompareResult::Diff { left, right } => {
                    if right.len() > 0 {
                        return Err(
                            "no fast forward, remote has commits not in the current db".into()
                        );
                    }
                }
            }

            println!("Uploading database");
            upload_object(
                &client,
                bucket,
                &remote_directory.join(".sssync/sssync.db"),
                &root_path.join(".sssync/sssync.db"),
            )
            .await?;

            let tree = db::tree::get_tree(connection, &head.hash)?;
            let hashes = tree.iter().map(|t| t.file_hash.to_string()).collect();
            println!("Saving migration");
            let migration =
                crate::migration::create(connection, MigrationKind::Upload, &remote.name, &hashes)?;

            println!("Running Migration");
            crate::migration::run(connection, root_path, &migration).await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}
