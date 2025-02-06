use std::path::Path;

use anyhow::{anyhow, Result};
use rusqlite::Connection;

use crate::db::staging;
use crate::models::status::Status;
use crate::store;

/* Either unstages everything that's staged or if --hard is passed will reset
 * the current file state to the latest commit in head
 */
pub fn reset(
    connection: &Connection,
    root_path: &Path,
    hard: bool,
) -> Result<()> {
    staging::delete(connection)?;
    if hard {
        let status = Status::new(connection, root_path)?;
        for ua in status.unstaged_additions {
            let tf = status
                .tracked_files
                .get(&ua)
                .ok_or(anyhow!("weird, missing tracked file"))?;

            let full_file_path = root_path.join(&ua);
            store::export_to(root_path, &tf.file_hash, &full_file_path)?;
        }

        for ud in status.unstaged_deletions {
            let tf = status
                .tracked_files
                .get(&ud)
                .ok_or(anyhow!("weird, missing tracked file"))?;

            let full_file_path = root_path.join(&ud);
            store::export_to(root_path, &tf.file_hash, &full_file_path)?;
        }
    }
    Ok(())
}
