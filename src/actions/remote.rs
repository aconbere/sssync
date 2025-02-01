use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::db::tree::diff_all_commits;
use crate::helpers::{bucket_from_url, strip_leading_slash};
use crate::models::commit;
use crate::models::reference;
use crate::models::remote;
use crate::models::remote::Remote;
use crate::models::transfer::TransferKind;
use crate::models::tree_file::TreeFile;
use crate::s3::make_client;
use crate::s3::upload_multipart::upload_multipart;
use crate::store;
use crate::types::remote_kind::RemoteKind;

// Add a remote to the repository
pub fn add(
    connection: &Connection,
    name: &str,
    kind: &RemoteKind,
    location: &str,
) -> Result<()> {
    let remote = Remote::new(name, *kind, location)?;
    db::remote::insert(connection, &remote)
}

// Remote a remote from the repository
pub fn remove(connection: &Connection, name: &str) -> Result<()> {
    db::remote::delete(connection, name).map_err(|e| e.into())
}

// Locate a named file in the remote. Returns the URL that references the file.
pub fn locate(
    connection: &Connection,
    remote_name: &str,
    path: &Path,
) -> Result<Url> {
    let remote = db::remote::get(connection, remote_name)?;
    let tree_file = db::tree::get_by_path(connection, path)?;
    let url =
        remote::remote_object_path(&remote.location, &tree_file.file_hash)?;
    println!("Url: {}", url);
    Ok(url)
}

// List the remotes in the repository
pub fn list(connection: &Connection) -> Result<()> {
    let remotes = db::remote::get_all(connection)?;

    for remote in remotes {
        println!("Remote: {} {}", remote.name, remote.location);
    }

    Ok(())
}

/* Initialize the remote
 *
 * Because sssync doesn't run a remote daemon nor expect remote ssh access it
 * might need to coordinate the set up of the remote. As an example if the
 * ssync backend is s3 ssync might need to setup the bucket and push up an
 * initial database.
 *
 * If the given remote url is already a sssync repository (contains a
 * .sssync.db) then this function will exit with Ok skipping any mutations.
 * Pass `true` to `force` in order to overwrite the existing repository.
 */
pub async fn init(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
    force: bool,
) -> Result<()> {
    let remote = db::remote::get(connection, remote_name)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("No commit"))?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = bucket_from_url(&u)?;

            let remote_directory = Path::new(u.path());
            let remote_db_path = remote_directory.join(".sssync/sssync.db");

            // check if the remote file exists before running init
            let head_object_res = client
                .head_object()
                .bucket(&bucket)
                .key(strip_leading_slash(remote_db_path.to_str().ok_or(
                    anyhow!("unable to unwrap remote_db_path to string"),
                )?))
                .send()
                .await;

            if head_object_res.is_ok() && !force {
                println!("WARNING: remote already exists.");
                println!("\trun with --force to init anyway.");
                return Ok(());
            }

            let local_db_path = store::db_path(root_path);

            let tree = db::tree::get(connection, &head.hash)?;
            let hashes: Vec<String> =
                tree.iter().map(|t| t.file_hash.to_string()).collect();

            let migration = crate::migration::create(
                connection,
                TransferKind::Upload,
                &remote.name,
                &hashes,
            )?;

            println!("Running Migration");
            crate::migration::run(
                connection, root_path, &migration, force, true,
            )
            .await?;

            upload_multipart(
                &client,
                &bucket,
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
) -> Result<()> {
    let remote = db::remote::get(connection, remote_name)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or(anyhow!("No commit"))?;

    match remote.kind {
        RemoteKind::S3 => {
            let s3_client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = bucket_from_url(&u)?;
            let remote_directory = Path::new(u.path());

            // database isn't right remotely so this updates with the
            // bad copy.
            let remote_db = crate::remote::fetch_remote_database(
                &s3_client,
                root_path,
                remote.kind,
                &remote.name,
                &remote.location,
            )
            .await?;

            let remote_connection = Connection::open(&remote_db)?;
            let remote_head =
                db::commit::get_by_ref_name(&remote_connection, &meta.head)?
                    .ok_or(anyhow!("No remote commit"))?;
            println!(
                "Updating {} from {} to {}...",
                remote_name, remote_head.hash, head.hash
            );

            if remote_head.hash == head.hash {
                return Err(anyhow!("Remote is already at: {}", head.hash));
            }

            // figure out what commits are different
            let local_commits =
                db::commit::get_children(connection, &head.hash)?;

            let remote_commits = db::commit::get_children(
                &remote_connection,
                &remote_head.hash,
            )?;

            let ff_commits =
                commit::diff_commit_list_left(&local_commits, &remote_commits)?;

            // figure out what files to upload
            let diff = diff_all_commits(&connection, &ff_commits)?;

            let mut updated_files: Vec<TreeFile> = diff.additions.clone();
            updated_files.extend(diff.changes.clone());

            let to_upload_hashes: Vec<String> =
                updated_files.iter().map(|f| f.file_hash.clone()).collect();

            let migration = crate::migration::create(
                connection,
                TransferKind::Upload,
                &remote.name,
                &to_upload_hashes,
            )?;

            println!("Running Migration");
            crate::migration::run(
                connection, root_path, &migration, false, true,
            )
            .await?;

            for commit in ff_commits {
                db::commit::insert(&remote_connection, &commit)?;
            }
            db::reference::update(
                &remote_connection,
                &meta.head,
                reference::Kind::Branch,
                &head.hash,
                None,
            )?;
            remote_connection.close().unwrap();

            upload_multipart(
                &s3_client,
                &bucket,
                &remote_directory.join(".sssync/sssync.db"),
                &store::remote_db_path(root_path, remote_name),
                true,
            )
            .await?;

            println!("done uploading");

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

/* Pull down the remote database
 *
 * This will not fetch down remote objects. Bceause a goal of sssync is to
 * minimize transfer costs its useful to have a distinction between getting
 * the latest remote state (fetch) and getting the relevant remote objects
 * (undefined as of yet).
 */
pub async fn fetch_remote_database(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<()> {
    let remote = db::remote::get(connection, remote_name)?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;

            let remote_path = crate::remote::fetch_remote_database(
                &client,
                root_path,
                remote.kind,
                &remote.name,
                &remote.location,
            )
            .await?;

            let remote_connection = Connection::open(remote_path)?;

            // Upate the local database with the state of the remote database
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

/* Push up a copy of the remote database. This probably shouldn't be used,
 * but it allows me to easily fix up a remote db.
 */
pub async fn push_remote_database(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
    force: bool,
) -> Result<()> {
    let remote = db::remote::get(connection, remote_name)?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;

            let local_db_path = store::remote_db_path(root_path, remote_name);

            let u = Url::parse(&remote.location)?;
            let bucket = bucket_from_url(&u)?;
            let remote_directory = Path::new(u.path());
            let remote_db_path = remote_directory.join(".sssync/sssync.db");

            println!(
                "Uploading: {} to {}",
                local_db_path.display(),
                remote_db_path.display()
            );

            upload_multipart(
                &client,
                &bucket,
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
