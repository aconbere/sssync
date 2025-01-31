use std::path::Path;

use anyhow::Result;
use rusqlite::Connection;

use crate::db;
use crate::models::staged_file;
use crate::models::status::Status;
use crate::store;

pub fn add(
    connection: &Connection,
    root_path: &Path,
    rel_path: &Path,
) -> Result<()> {
    let status = Status::new(connection, root_path)?;

    for ua in status.unstaged_additions {
        if ua.starts_with(rel_path) {
            let full_file_path = root_path.join(&ua);
            println!("staging addition: {}", full_file_path.display());

            let staged_file =
                staged_file::StagedFile::new(&full_file_path, &ua)?;

            store::insert_from(
                root_path,
                &staged_file.file_hash,
                &full_file_path,
            )?;

            db::staging::insert(
                connection,
                &staged_file::Change::Addition(staged_file),
            )?;
        }
    }

    for ua in status.unstaged_deletions {
        if ua.starts_with(rel_path) {
            let full_file_path = root_path.join(&ua);
            println!("staging deletion: {}", full_file_path.display());
            db::staging::insert(
                connection,
                &staged_file::Change::Deletion(ua),
            )?;
        }
    }
    Ok(())
}
