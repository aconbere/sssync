use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;
use bytes::BytesMut;

use crate::s3::upload::upload_object;

const TEN_MEGABYTES: u64 = 10_000_000;
const FIVE_MEGABYTES: u64 = 5_000_000;

pub async fn upload_multipart(
    client: &Client,
    bucket: &str,
    key_path: &Path,
    file_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let key = key_path.to_str().unwrap();

    let mut file = File::open(file_path)?;

    let metadata = file.metadata()?;

    // AWS S3 Multipart Uploads don't work with files
    // less than 5Mb, if we catch this case, just do
    // a simple file upload
    if metadata.len() < FIVE_MEGABYTES {
        return upload_object(client, bucket, key_path, file_path)
            .await
            .map_err(|e| e.into());
    }

    let multipart = client
        .create_multipart_upload()
        .bucket(bucket)
        .key(key)
        .send()
        .await?;

    let upload_id = multipart.upload_id.unwrap_or(String::from("no upload id"));

    let result = run(client, &upload_id, bucket, key_path, &mut file).await;

    if result.is_err() {
        client
            .abort_multipart_upload()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;
    }

    Ok(())
}

async fn run(
    client: &Client,
    upload_id: &str,
    bucket: &str,
    _key_path: &Path,
    reader: &mut dyn Read,
) -> Result<(), Box<dyn Error>> {
    let mut part_number = 1;

    loop {
        let mut buf = BytesMut::with_capacity(TEN_MEGABYTES as usize);
        let read_bytes = reader.read(&mut buf)?;

        if read_bytes == 0 {
            return Ok(());
        }

        let a = buf.freeze();
        let b = a.clone();

        client
            .upload_part()
            .bucket(bucket)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(ByteStream::from(b))
            .send()
            .await?;

        part_number += 1;
    }
}
