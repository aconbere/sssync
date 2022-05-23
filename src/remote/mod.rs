use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use url::Url;

use crate::models::remote::Remote;
use crate::s3::download_object;
use crate::store;
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
            let db_path = directory.join(".sssync/sssync.db");
            download_object(&client, bucket, db_path.to_str().unwrap(), writer).await?;
            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn fetch_remote_database(
    client: &Client,
    remote: &Remote,
    root_path: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    match remote.kind {
        RemoteKind::S3 => {
            let remote_db_copy_path = store::store_path(&root_path)
                .join(store::REMOTES_DIR)
                .join(format!("{}.db", &remote.name));

            println!("fetching db into: {}", &remote_db_copy_path.display());
            let mut remote_db_copy_file = File::create(&remote_db_copy_path)?;
            fetch_database(&client, &remote, &mut remote_db_copy_file).await?;
            Ok(remote_db_copy_path)
        }
        RemoteKind::Local => Ok(root_path.to_path_buf()),
    }
}
