use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Result;
use aws_sdk_s3::Client;
use url::Url;

use crate::helpers::bucket_from_url;
use crate::s3::download_object;
use crate::store;
use crate::types::remote_kind::RemoteKind;

pub async fn fetch_remote_database(
    client: &Client,
    root_path: &Path,
    remote_kind: RemoteKind,
    remote_name: &str,
    remote_location: &str,
) -> Result<PathBuf> {
    match remote_kind {
        RemoteKind::S3 => {
            let destination_path =
                store::remote_db_path(root_path, remote_name);
            let mut destination_file = File::create(&destination_path)?;

            let url = Url::parse(remote_location)?;
            let bucket = bucket_from_url(&url)?;
            let key = Path::new(url.path()).join(".sssync/sssync.db");

            download_object(client, &bucket, &key, &mut destination_file)
                .await?;

            Ok(destination_path)
        }
        RemoteKind::Local => Ok(root_path.to_path_buf()),
    }
}
