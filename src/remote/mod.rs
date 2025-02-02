use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Result};
use aws_sdk_s3::Client;
use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::migration;
use crate::models::transfer::TransferKind;
use crate::s3;
use crate::store;
use crate::types::remote_kind::RemoteKind;

pub struct RemoteInfo {
    pub bucket: String,
    pub prefix: String,
    pub kind: RemoteKind,
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
        let scheme = u.scheme();
        let kind = RemoteKind::parse(scheme)?;

        // for a url like `s3://anders.conbere.org/games` the url decomposes to
        // bucket: anders.conbere.org
        // key: /games
        let bucket = u
            .host_str()
            .ok_or(anyhow!("unable to unwrap host to bucket"))?
            .to_string();

        let mut prefix = u.path().to_string();
        prefix.remove(0);

        Ok(Self {
            bucket,
            prefix,
            kind,
        })
    }
}

pub async fn fetch_remote_db(
    client: &Client,
    remote_info: &RemoteInfo,
    destination: &Path,
) -> Result<()> {
    let mut remote_db = File::create(&destination)?;

    s3::download_object(
        &client,
        &remote_info.bucket,
        &remote_info.database_key(),
        &mut remote_db,
    )
    .await?;

    Ok(())
}

pub async fn fetch_remote_objects(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<()> {
    let remote = db::remote::get(connection, remote_name)?;

    match remote.kind {
        RemoteKind::S3 => {
            let meta = db::meta::get(&connection)?;

            let head = db::commit::get_by_ref_name(&connection, &meta.head)?
                .ok_or(anyhow!("Head is bad - no matching ref name"))?;

            let files = db::tree::get(&connection, &head.hash)?;

            let object_hashes: Vec<String> =
                files.iter().map(|f| f.file_hash.clone()).collect();

            let migration = migration::create(
                &connection,
                TransferKind::Download,
                remote_name,
                &object_hashes,
            )?;

            migration::run(connection, root_path, &migration, false, true)
                .await?;

            for f in files {
                let p = &root_path.join(f.path);
                store::export_to(&root_path, &f.file_hash, p)?;
            }

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}
