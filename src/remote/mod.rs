use std::error::Error;

use url::Url;

use crate::models::remote::Remote;
use crate::types::remote_kind::RemoteKind;

pub fn download(remote: &Remote) -> Result<(), Box<dyn Error>> {
    match remote.kind {
        RemoteKind::S3 => {
            download_s3(&remote.location)?;
        }
        RemoteKind::Local => {}
    }

    let remote_url = Url::parse(&remote.location)?;
    match remote_url.scheme() {
        "s3" => {
            println!("path:{}", remote_url.path());
            Ok(())
        }
        "file" => {
            println!("path:{}", remote_url.path());
            match remote_url.host() {
                Some(h) => println!("host:{}", h),
                None => {}
            }
            Ok(())
        }
        _ => Err(String::from("invalid url scheme").into()),
    }
}

pub fn download_s3(location: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}
