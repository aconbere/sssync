use std::error::Error;
use std::path::Path;

use rusqlite::Connection;

use crate::db;
use crate::models::staged_file;
use crate::models::status::Status;
use crate::store;

pub fn add(
    connection: &Connection,
    root_path: &Path,
    rel_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if rel_path == Path::new("") {
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
            db::staging::insert(
                connection,
                &staged_file::Change::Deletion(ua),
            )?;
        }
        return Ok(());
    }

    let staged_file =
        staged_file::StagedFile::new(&root_path.join(&rel_path), &rel_path)?;
    store::insert_from(root_path, &staged_file.file_hash, &rel_path)?;
    db::staging::insert(
        connection,
        &staged_file::Change::Addition(staged_file),
    )?;

    Ok(())
}
