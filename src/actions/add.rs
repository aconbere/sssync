use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::models::staged_file;
use crate::models::status::Status;
use crate::store;

pub fn add(
    connection: &Connection,
    full_path: &Path,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?;

    let status = Status::new(connection, root_path)?;

    for ua in status.unstaged_additions {
        let staged_file =
            staged_file::StagedFile::new(&root_path.join(&ua), &ua)?;
        store::insert_from(root_path, &staged_file.file_hash, &ua)?;
        db::staging::insert(
            connection,
            &staged_file::Change::Addition(staged_file),
        )?;
    }

    for ua in status.unstaged_deletions {
        db::staging::insert(connection, &staged_file::Change::Deletion(ua))?;
    }

    Ok(())
}
