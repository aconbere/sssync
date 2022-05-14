use std::error::Error;
use std::path::Path;

use crate::db;
use crate::store;

pub fn init(root_path: &Path) -> Result<(), Box<dyn Error>> {
    println!("init: {}", root_path.display());
    store::init(&root_path)?;

    let connection = db::get_connection(&root_path)?;
    println!("found connection: {}", root_path.display());
    db::init(&connection)?;
    Ok(())
}
