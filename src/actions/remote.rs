use std::error::Error;
use std::path::Path;

use rusqlite::Connection;
use url::Url;

use crate::db;
use crate::models::commit;
use crate::models::migration::MigrationKind;
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

pub fn remove(connection: &Connection, name: &str) -> Result<(), Box<dyn Error>> {
    db::remote::delete(connection, name).map_err(|e| e.into())
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
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?.ok_or("No commit")?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);

            let remote_db_path = remote_directory.join(".sssync/sssync.db");
            let local_db_path = root_path.join(".sssync/sssync.db");

            let tree = db::tree::get(connection, &head.hash)?;
            let hashes = tree.iter().map(|t| t.file_hash.to_string()).collect();

            println!("Saving migration");
            let migration =
                crate::migration::create(connection, MigrationKind::Upload, &remote.name, &hashes)?;

            println!("Running Migration");
            crate::migration::run(connection, root_path, &migration).await?;

            println!("Uploading database");
            upload_multipart(&client, bucket, &remote_db_path, &local_db_path).await?;

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
    let head = db::commit::get_by_ref_name(connection, &meta.head)?.ok_or("No commit")?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            let u = Url::parse(&remote.location)?;

            let bucket = u.host_str().unwrap();
            let remote_directory = Path::new(u.path()).join(&remote.name);

            let remote_db_copy =
                crate::remote::fetch_remote_database(&client, &remote, &root_path).await?;

            let remote_db_connection = Connection::open(&remote_db_copy)?;
            let remote_head = db::commit::get_by_ref_name(&remote_db_connection, &meta.head)?
                .ok_or("No remote commit")?;

            if remote_head.hash == head.hash {
                return Err("no differences between remote and local".into());
            }

            // Note: In order for commits to actually work we need to push not just
            // the current commits changes, but also all the intermediate commits.
            //
            // This is important because there's no other way to catch up a peer.
            //
            // Example:
            //
            // A: 1 -> 2 -> 3
            // B: 1 -> 2
            // C: 1
            //

            let commits = db::commit::get_all(connection, &head.hash)?;
            let remote_commits = db::commit::get_all(&remote_db_connection, &remote_head.hash)?;

            let fast_forward_commits: Result<Vec<commit::Commit>, Box<dyn Error>> =
                match commit::diff_commit_list(&commits, &remote_commits) {
                    commit::CompareResult::NoSharedParent => {
                        Err("Remote has no shared parent".into())
                    }
                    commit::CompareResult::Diff { left, right } => {
                        if right.len() > 0 {
                            Err("no fast forward, remote has commits not in the current db".into())
                        } else if left.len() == 0 {
                            Err("no differences between remote and local".into())
                        } else {
                            Ok(left)
                        }
                    }
                };

            let _fast_forward_commits = fast_forward_commits?;

            let diff = db::tree::diff(connection, &head, &remote_head)?;
            println!("found diff: {:?}", diff);
            let files_to_upload = [diff.additions, diff.changes].concat();
            let hashes = files_to_upload
                .iter()
                .map(|d| d.file_hash.to_string())
                .collect();
            println!("found {} additional files to uplaod", files_to_upload.len());

            println!("Creating migration");
            let migration =
                crate::migration::create(connection, MigrationKind::Upload, &remote.name, &hashes)?;

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
            )
            .await?;

            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}

pub async fn fetch(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;
    //let meta = db::meta::get(connection)?;

    //let head = db::commit::get_by_ref_name(connection, &meta.head)?.ok_or("No commit")?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            // let u = Url::parse(&remote.location)?;
            //let bucket = u.host_str().unwrap();
            //let remote_directory = Path::new(u.path()).join(&remote.name);

            crate::remote::fetch_remote_database(&client, &remote, &root_path).await?;
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
#[allow(dead_code)]
pub async fn sync(
    connection: &Connection,
    root_path: &Path,
    remote_name: &str,
) -> Result<(), Box<dyn Error>> {
    let remote = db::remote::get(connection, remote_name)?;

    match remote.kind {
        RemoteKind::S3 => {
            let client = make_client().await;
            crate::remote::fetch_remote_database(&client, &remote, &root_path).await?;
            Ok(())
        }
        RemoteKind::Local => Ok(()),
    }
}
