use std::io;
use std::path::Path;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::model::{Delete, ObjectIdentifier};
use aws_sdk_s3::output::ListObjectsV2Output;
use aws_sdk_s3::types::ByteStream;
use aws_sdk_s3::{Client, Error};
use tokio_stream::StreamExt;

mod upload_multipart;

pub async fn make_client() -> Client {
    let region_provider = RegionProviderChain::default_provider().or_else("us-west-2");
    let config = aws_config::from_env().region(region_provider).load().await;
    Client::new(&config)
}

pub async fn download_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
    writer: &mut dyn io::Write,
) -> Result<ByteStream, Box<dyn std::error::Error>> {
    println!("downloading object: {} {}", bucket_name, key);
    let mut resp = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await?;

    while let Some(bytes) = resp.body.try_next().await? {
        writer.write(&bytes)?;
    }

    Ok(resp.body)
}

pub async fn upload_object(
    client: &Client,
    bucket_name: &str,
    file_path: &Path,
    key: &Path,
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

pub async fn list_objects(client: &Client, bucket_name: &str) -> Result<(), Error> {
    let objects = client.list_objects_v2().bucket(bucket_name).send().await?;
    for obj in objects.contents().unwrap_or_default() {
        println!("{:?}", obj.key().unwrap());
    }

    Ok(())
}

pub async fn delete_objects(client: &Client, bucket_name: &str) -> Result<(), Error> {
    let objects = client.list_objects_v2().bucket(bucket_name).send().await?;

    let mut delete_objects: Vec<ObjectIdentifier> = vec![];
    for obj in objects.contents().unwrap_or_default() {
        let obj_id = ObjectIdentifier::builder()
            .set_key(Some(obj.key().unwrap().to_string()))
            .build();
        delete_objects.push(obj_id);
    }
    client
        .delete_objects()
        .bucket(bucket_name)
        .delete(Delete::builder().set_objects(Some(delete_objects)).build())
        .send()
        .await?;

    let objects: ListObjectsV2Output = client.list_objects_v2().bucket(bucket_name).send().await?;
    match objects.key_count {
        0 => Ok(()),
        _ => Err(Error::Unhandled(Box::from(
            "There were still objects left in the bucket.",
        ))),
    }
}
