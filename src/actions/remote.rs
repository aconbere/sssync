use std::error::Error;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::models::commit;
use crate::models::migration::MigrationKind;
use crate::models::reference;
use crate::models::remote;
use crate::models::remote::Remote;
use crate::s3::make_client;
use crate::s3::upload_multipart::upload_multipart;
use crate::types::remote_kind::RemoteKind;

pub fn add(
    connection: &Connection,
    name: &str,
    kind: &RemoteKind,
    location: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = Remote::new(name, kind.clone(), location)?;
    db::remote::insert(connection, &remote)
}

pub fn remove(
    connection: &Connection,
    name: &str,
) -> Result<(), Box<dyn Error>> {
    db::remote::delete(connection, name).map_err(|e| e.into())
}

pub fn locate(
    connection: &Connection,
    remote_name: &str,
    path: &Path,
) -> Result<Url, Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let tree_file = db::tree::get_by_path(connection, path)?;
    let url =
        remote::remote_object_path(&remote.location, &tree_file.file_hash)?;
    println!("Url: {}", url);
    Ok(url)
}

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let remotes = db::remote::get_all(connection)?;

    for remote in remotes {
        println!("Remote: {} {}", remote.name, remote.location);
    }

    Ok(())
}

/* Initialize the remote
 *
 * Because sssync doesn't run a remote daemon nor expect remote ssh access it might need to
 * coordinate the set up of the remote. As an example if the ssync backend is s3 ssync might need
 * to setup the bucket and push up an initial database.
 */
pub async fn init(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
    force: bool,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or("No commit")?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path());

            let remote_db_path = remote_directory.join(".sssync/sssync.db");

            // check if the remote file exists before running init
            let head_object_res = client
                .head_object()
                .bucket(bucket)
                .key(remote_db_path.to_str().unwrap())
                .send()
                .await;

            if head_object_res.is_ok() {
                if !force {
                    println!("WARNING: remote already exists.");
                    println!("\trun with --force to init anyway.");
                    return Ok(());
                }
            }

            let local_db_path = root_path.join(".sssync/sssync.db");

            let tree = db::tree::get(connection, &head.hash)?;
            let hashes = tree.iter().map(|t| t.file_hash.to_string()).collect();

            let migration = crate::migration::create(
                connection,
                MigrationKind::Upload,
                &remote.name,
                &hashes,
            )?;

            println!("Running Migration");
            crate::migration::run(connection, root_path, &migration).await?;

            upload_multipart(
                &client,
                bucket,
                &remote_db_path,
                &local_db_path,
                force,
            )
            .await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn push(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or("No commit")?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);

            let remote_db_copy = crate::remote::fetch_remote_database(
                &client, &remote, &root_path,
            )
            .await?;

            let remote_db_connection = Connection::open(&remote_db_copy)?;
            let remote_head =
                db::commit::get_by_ref_name(&remote_db_connection, &meta.head)?
                    .ok_or("No remote commit")?;

            if remote_head.hash == head.hash {
                return Err("no differences between remote and local".into());
            }

            let commits = db::commit::get_children(connection, &head.hash)?;
            let remote_commits = db::commit::get_children(
                &remote_db_connection,
                &remote_head.hash,
            )?;

            let ff_commits =
                commit::diff_commit_list_left(&commits, &remote_commits)?;
            let additions = db::tree::additions(connection, &ff_commits)?;

            let hashes =
                additions.iter().map(|d| d.file_hash.to_string()).collect();

            let migration = crate::migration::create(
                connection,
                MigrationKind::Upload,
                &remote.name,
                &hashes,
            )?;

            println!("Running Migration");
            crate::migration::run(connection, root_path, &migration).await?;

            println!(
                "Uploading database from {} to {}",
                &remote_directory.display(),
                &root_path.display()
            );
            upload_multipart(
                &client,
                bucket,
                &remote_directory.join(".sssync/sssync.db"),
                &root_path.join(".sssync/sssync.db"),
                true,
            )
            .await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

/* Pull down the remote database
 *
 * This will not fetch down remote objects. Bceause a goal of sssync is to minimize transfer costs
 * its useful to have a distinction between getting the latest remote state (fetch) and getting the
 * relevant remote objects (undefined as of yet).
 */
pub async fn fetch_remote_database(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;

            let remote_path = crate::remote::fetch_remote_database(
                &client, &remote, &root_path,
            )
            .await?;

            let remote_connection = Connection::open(remote_path)?;

            println!("Adding commits");
            let remote_commits = db::commit::get_all(&remote_connection)?;
            for commit in remote_commits {
                db::commit::insert(connection, &commit)?;
            }

            println!("Adding refs");
            let remote_refs = db::reference::get_all_by_kind(
                &remote_connection,
                None,
                reference::Kind::Branch,
            )?;
            for _ref in remote_refs {
                db::reference::insert(
                    connection,
                    &_ref.name,
                    _ref.kind,
                    &_ref.hash,
                    Some(remote_name),
                )?;
            }

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}
