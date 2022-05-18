use std::error::Error;
use std::io::Write;

use url::Url;

use crate::models::remote::Remote;
use crate::s3::download_object;
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
            fetch_database_s3(client, &u.host_str().unwrap(), writer).await?;
            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn fetch_database_s3(
    client: &Client,
    bucket: &str,
    writer: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    download_object(&client, bucket, "./sssync/sssync.db", writer).await?;
    Ok(())
}
