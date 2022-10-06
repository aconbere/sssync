use std::error::Error;
use std::path::Path;

use crate::models::status::Status;
use rusqlite::Connection;

/* The goal of status is to compare three states:
 *  - The state of the store
 *  - The state of the index
 *  - The state of the filesystem
 *
 *  It does this by building up a set of each of these files (TreeFiles), and comparing
 *  the sets to produce a human readable string outpute.
 */
pub fn status(
    connection: &Connection,
    root_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let res = Status::new(connection, root_path)?;
    print!("{}", res);
    Ok(())
}
