use std::error::Error;

use url::Url;

use crate::models::remote::Remote;

pub fn download(remote: &Remote) -> Result<(), Box<dyn Error>> {
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
