use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::s3::make_client;
use crate::types::remote_kind::RemoteKind;

pub async fn update(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;

            crate::remote::fetch_remote_database(
                &client,
                &root_path,
                remote.kind,
                &remote.name,
                &remote.location,
            )
            .await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}