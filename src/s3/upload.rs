use std::path::Path;

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Error};

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    key: &Path,
    file_path: &Path,
) -> Result<(), Error> {
    let body = ByteStream::from_path(file_path).await;
    client
        .put_object()
        .bucket(bucket_name)
        .key(key.to_str().unwrap())
        .body(body.unwrap())
        .send()
        .await?;

    Ok(())
}
