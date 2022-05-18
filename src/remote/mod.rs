use std::error::Error;
use std::io::Write;
use std::path::Path;

use url::Url;

use crate::models::remote::Remote;
use crate::s3::{download_object, upload_object};
use crate::types::remote_kind::RemoteKind;
use aws_sdk_s3::Client;

pub async fn fetch_database(
    client: &Client,
    remote: &Remote,
    writer: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    match remote.kind {
        RemoteKind::S3 => {
            let u = Url::parse(&remote.location)?;
            let bucket = u.host_str().unwrap();
            let directory = Path::new(u.path()).join(&remote.name);
            let db_path = directory.join("./sssync/sssync.db");
            download_object(&client, bucket, db_path.to_str().unwrap(), writer).await?;
            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn init(
    client: &Client,
    remote: &Remote,
    root_path: &Path,
) -> Result<(), Box<dyn Error>> {
    match remote.kind {
        RemoteKind::S3 => {
            let u = Url::parse(&remote.location)?;
            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);
            let remote_db_path = remote_directory.join(".sssync/sssync.db");
            let local_db_path = root_path.join(".sssync/sssync.db");
            upload_object(
                client,
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
