use std::path::Path;

use crate::helpers::strip_leading_slash;
use anyhow::{anyhow, Result};
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    key: &Path,
    file_path: &Path,
) -> Result<()> {
    let body = ByteStream::from_path(file_path).await?;
    let key_str = strip_leading_slash(
        key.to_str()
            .ok_or(anyhow!("could not convert key to string"))?,
    );

    client
        .put_object()
        .bucket(bucket_name)
        .key(key_str)
        .body(body)
        .send()
        .await?;

    Ok(())
}
