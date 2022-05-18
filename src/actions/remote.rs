use std::error::Error;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use crate::db;
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
    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;
            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);
            let remote_db_path = remote_directory.join(".sssync/sssync.db");
            let local_db_path = root_path.join(".sssync/sssync.db");
            upload_object(
                &client,
                bucket,
                &local_db_path,
                &remote_db_path.to_str().unwrap(),
            )
            .await?;
            // at this point we need to kick off a migration
            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}
