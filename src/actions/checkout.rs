use std::error::Error;

use rusqlite::Connection;

use crate::db;
use crate::tree::TreeDiff;

pub fn checkout(
    connection: &Connection,
    hash: &str,
) -> Result<(), Box<dyn Error>> {
    let staged_files = db::staging::get_all(connection)?;

    if !staged_files.is_empty() {
        return Err("There are currently staged files: Commit your current work or reset your state to continue".into());
    }

    let meta = db::meta::get(connection)?;
    let head = db::commit::get_by_ref_name(connection, &meta.head)?
        .ok_or("Head is bad - no matching ref name")?;

    let current_tree = db::tree::get(connection, &head.hash)?;
    let new_tree = db::tree::get(connection, hash)?;
    let diff = TreeDiff::new(&current_tree, &new_tree);

    for f in diff.additions {
        println!("Diff: {}", f.path)
    }

    Ok(())
}
