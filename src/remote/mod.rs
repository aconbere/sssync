use crate::s3;
use crate::store;
use anyhow::{anyhow, Result};
use aws_sdk_s3::Client;
use std::fs::File;
use std::path::{Path, PathBuf};
use url::Url;

pub struct RemoteInfo {
    pub bucket: String,
    pub prefix: String,
}

impl RemoteInfo {
    pub fn object_key(&self, hash: &str) -> String {
        format!("{}/.sssync/objects/{}", self.prefix, hash)
    }

    pub fn database_key(&self) -> String {
        format!("{}/.sssync/sssync.db", self.prefix)
    }

    pub fn from_url(url: &str) -> Result<Self> {
        let u = Url::parse(url)?;
        // for a url like `s3://anders.conbere.org/games` the url decomposes to
        // bucket: anders.conbere.org
        // key: /games
        let bucket = u
            .host_str()
            .ok_or(anyhow!("unable to unwrap host to bucket"))?
            .to_string();

        let mut prefix = u.path().to_string();
        prefix.remove(0);

        Ok(Self { bucket, prefix })
    }
}

pub async fn fetch_remote_db(
    client: &Client,
    root_path: &Path,
    remote_name: &str,
    remote_info: &RemoteInfo,
) -> Result<PathBuf> {
    let remote_db_path = store::remote_db_path(root_path, remote_name);
    let mut remote_db = File::create(&remote_db_path)?;

    s3::download_object(
        &client,
        &remote_info.bucket,
        &remote_info.database_key(),
        &mut remote_db,
    )
    .await?;

    Ok(remote_db_path)
}
