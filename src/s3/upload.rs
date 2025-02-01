use std::path::Path;

use anyhow::Result;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::Client;

/* Simple single file upload function
 */
pub async fn upload_object(
    client: &Client,
    bucket: &str,
    key: &str,
    file_path: &Path,
) -> Result<()> {
    let body = ByteStream::from_path(file_path).await?;

    client
        .put_object()
        .bucket(bucket)
        .key(key)
        .body(body)
        .send()
        .await?;

    Ok(())
}
