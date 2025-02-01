use anyhow::{anyhow, Result};
use url::Url;

pub fn bucket_from_url(u: &Url) -> Result<String> {
    let bucket = u
        .host_str()
        .ok_or(anyhow!("unable to unwrap host to bucket"))?
        .to_string();
    Ok(bucket)
}

pub fn strip_leading_slash(s: &str) -> String {
    let mut new_path = s.to_string();
    new_path.remove(0);
    new_path
}
