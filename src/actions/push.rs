use std::error::Error;
use std::fs::File;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::remote;
use crate::s3::make_client;
use crate::store;

pub async fn push(
    connection: &Connection,
    root_path: &Path,
    remote: &str,
) -> Result<(), Box<dyn Error>> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?.ok_or("No commit")?;
    let remote = db::remote::get(connection, remote)?;

    println!(
        "Pushing {} to remote: {} {} {}",
        head.hash, remote.name, remote.kind, remote.location
    );

    let output_file_path = store::store_path(&root_path)
        .join(store::REMOTES_DIR)
        .join(format!("{}.db", &remote.name));

    let client = make_client().await;
    println!("fetching db into: {}", &output_file_path.display());

    let mut output_file = File::create(&output_file_path)?;
    remote::fetch_database(&client, &remote, &mut output_file).await?;

    // now run a migration

    Ok(())
}
