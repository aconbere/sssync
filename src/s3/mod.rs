use std::io;

use anyhow::{anyhow, Result};
use aws_sdk_s3::operation::list_objects_v2::ListObjectsV2Output;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{Delete, ObjectIdentifier};
use aws_sdk_s3::Client;

pub mod upload;
pub mod upload_multipart;

pub async fn make_client() -> Client {
    let config =
        aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    Client::new(&config)
}

pub async fn download_object(
    client: &Client,
    bucket_name: &str,
    key: &str,
    writer: &mut dyn io::Write,
) -> Result<ByteStream> {
    let mut resp = client
        .get_object()
        .bucket(bucket_name)
        .key(key)
        .send()
        .await?;

    while let Some(bytes) = resp.body.try_next().await? {
        writer.write_all(&bytes)?;
    }

    Ok(resp.body)
}

#[allow(dead_code)]
pub async fn list_objects(client: &Client, bucket_name: &str) -> Result<()> {
    let objects = client.list_objects_v2().bucket(bucket_name).send().await?;

    if let Some(objects) = objects.contents {
        for obj in objects {
            println!("{:?}", obj.key().unwrap());
        }
    } else {
        println!("No objects found for bucket: {}", bucket_name)
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn delete_objects(client: &Client, bucket_name: &str) -> Result<()> {
    let objects = client.list_objects_v2().bucket(bucket_name).send().await?;

    let mut delete_objects: Vec<ObjectIdentifier> = vec![];

    if let Some(objects) = objects.contents {
        for obj in objects {
            let obj_id = ObjectIdentifier::builder()
                .set_key(Some(obj.key().unwrap().to_string()))
                .build()?;
            delete_objects.push(obj_id);
        }

        client
            .delete_objects()
            .bucket(bucket_name)
            .delete(
                Delete::builder()
                    .set_objects(Some(delete_objects))
                    .build()?,
            )
            .send()
            .await?;

        let objects: ListObjectsV2Output =
            client.list_objects_v2().bucket(bucket_name).send().await?;

        match objects.key_count {
            Some(0) => Ok(()),
            None => {
                Err(anyhow!("No objects found for bucket: {}", bucket_name))
            }
            _ => Err(anyhow!("There were still objects left in the bucket.")),
        }
    } else {
        Err(anyhow!("No objects found for bucket: {}", bucket_name))
    }
}
