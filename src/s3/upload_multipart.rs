use std::error::Error;
use std::fs::File;
use std::io::stdout;
use std::io::Read;
use std::io::Write;
use std::path::Path;

use aws_sdk_s3::model::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::Client;

use crate::s3::upload::upload_object;

const TEN_MEGABYTES: u64 = 10_000_000;
const FIVE_MEGABYTES: u64 = 5_000_000;

pub async fn upload_multipart(
    client: &Client,
    bucket: &str,
    key_path: &Path,
    file_path: &Path,
    check_first: bool,
) -> Result<(), Box<dyn Error>> {
    let key = key_path.to_str().unwrap();

    if check_first {
        let head_object_res =
            client.head_object().bucket(bucket).key(key).send().await;
        if head_object_res.is_ok() {
            return Ok(());
        }
    }

    let mut file = File::open(file_path)?;

    let file_metadata = file.metadata()?;

    // AWS S3 Multipart Uploads don't work with files
    // less than 5Mb, if we catch this case, just do
    // a simple file upload
    if file_metadata.len() < FIVE_MEGABYTES {
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

    let result = run(client, &upload_id, bucket, key, &mut file).await;

    match result {
        Err(e) => {
            println!("upload_multipart: upload failed: {}", e);
            client
                .abort_multipart_upload()
                .bucket(bucket)
                .key(key)
                .upload_id(upload_id)
                .send()
                .await?;
        }

        Ok(upload_parts) => {
            let completed_multipart_upload: CompletedMultipartUpload =
                CompletedMultipartUpload::builder()
                    .set_parts(Some(upload_parts))
                    .build();

            client
                .complete_multipart_upload()
                .multipart_upload(completed_multipart_upload)
                .bucket(bucket)
                .key(key)
                .upload_id(upload_id)
                .send()
                .await?;
        }
    }
    println!("\ndone");
    Ok(())
}

async fn run(
    client: &Client,
    upload_id: &str,
    bucket: &str,
    key: &str,
    reader: &mut dyn Read,
) -> Result<Vec<CompletedPart>, Box<dyn Error>> {
    let mut part_number = 1;

    let mut upload_parts: Vec<CompletedPart> = Vec::new();

    loop {
        print!(".");
        stdout().flush()?;

        let mut buf = vec![0u8; TEN_MEGABYTES as usize];
        let bytes_read = reader.read(&mut buf)?;

        if bytes_read == 0 {
            break;
        }

        let upload_part_res = client
            .upload_part()
            .key(key)
            .bucket(bucket)
            .upload_id(upload_id)
            .part_number(part_number)
            .body(ByteStream::from(buf))
            .send()
            .await?;

        upload_parts.push(
            CompletedPart::builder()
                .e_tag(upload_part_res.e_tag.unwrap_or_default())
                .part_number(part_number)
                .build(),
        );

        part_number += 1;
    }

    Ok(upload_parts)
}
