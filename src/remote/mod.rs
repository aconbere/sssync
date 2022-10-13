use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};

use aws_sdk_s3::Client;
use url::Url;

use crate::s3::download_object;
use crate::store;
use crate::types::remote_kind::RemoteKind;

pub async fn fetch_remote_database(
    client: &Client,
    root_path: &Path,
    remote_kind: RemoteKind,
    remote_name: &str,
    remote_location: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    match remote_kind {
        RemoteKind::S3 => {
            let copy_path = store::remote_db_path(root_path, remote_name);

            let mut copy_file = File::create(&copy_path)?;

            let url = Url::parse(remote_location)?;
            let bucket = url.host_str().unwrap();
            let directory = Path::new(url.path());
            let db_path = directory.join(".sssync/sssync.db");

            download_object(
                &client,
                bucket,
                db_path.to_str().unwrap(),
                &mut copy_file,
            )
            .await?;

            Ok(copy_path)
        }
        RemoteKind::Local => Ok(root_path.to_path_buf()),
    }
}
