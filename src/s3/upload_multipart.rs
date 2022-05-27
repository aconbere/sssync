use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::{Client, Error as S3Error};

pub struct MultipartUploader<T>
where
    T: Read,
{
    source: T,
}

impl<T> for MultipartUploader<T> where T:Read {
    pub fn new(source: T) -> Self {
        Self { source: source }
    }
}

pub async fn upload_multipart_object(
    client: &Client,
    bucket: &str,
    key: &Path,
    file_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let body = ByteStream::from_path(file_path).await;

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let multipart = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key.to_str().unwrap())
        .send()
        .await?;

    let upload_id = multipart.upload_id.unwrap_or(String::from("no uplaod id"));

    run(client, &upload_id, bucket, key.to_str().unwrap()).await?;

    Ok(())
}

async fn run(
    client: &Client,
    upload_id: &str,
    bucket: &str,
    key: &str,
) -> Result<(), Box<dyn Error>> {
    client
        .upload_part()
        .bucket(bucket)
        .upload_id(upload_id)
        .body(body.unwrap())
        .send()
        .await?;

    Ok(())
}
