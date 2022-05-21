use std::error::Error;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::models::migration::{Migration, MigrationKind};
use crate::models::remote::Remote;
use crate::models::upload::Upload;
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
            upload_object(&client, bucket, &local_db_path, &remote_db_path).await?;
            // at this point we need to kick off a migration

            let migration = Migration::new(MigrationKind::Upload, &remote);
            db::migration::insert(connection, &migration)?;

            let tree = db::tree::get_tree(connection, &head.hash)?;

            let uploads: Vec<Upload> = tree
                .iter()
                .map(|t| Upload::new(&migration.id, &t.file_hash))
                .collect();

            for upload in uploads {
                db::upload::insert(connection, &upload)?;
            }

            crate::migration::run(connection, root_path, &migration).await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}
