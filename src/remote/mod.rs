use std::error::Error;
use std::io::Write;
use std::path::Path;

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
            let directory = Path::new(u.path());
            fetch_database_s3(
                client,
                &remote.name,
                &u.host_str().unwrap(),
                &directory,
                writer,
            )
            .await?;
            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn fetch_database_s3(
    client: &Client,
    remote_name: &str,
    bucket: &str,
    directory: &Path,
    writer: &mut dyn Write,
) -> Result<(), Box<dyn Error>> {
    let object_path = directory.join(remote_name).join("./sssync/sssync.db");
    download_object(&client, bucket, object_path.to_str().unwrap(), writer).await?;
    Ok(())
}
