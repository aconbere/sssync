use rusqlite::params;
use rusqlite::Connection;
use std::error::Error;
use std::path::Path;

use crate::file_entry::FileEntry;

pub fn get_connection(path: &Path) -> Result<Connection, Box<dyn Error>> {
    let mut db_path = path.to_path_buf();
    db_path.push(".sssync.db");
    let connection = Connection::open(db_path.as_path())?;
    Ok(connection)
}

pub fn init_db(connection: Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "CREATE TABLE objects (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;

    connection.execute(
        "CREATE TABLE staging (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;

    connection.execute("CREATE TABLE commits (hash TEXT primary key)", params![])?;
    Ok(())
}

pub fn staging_get_all(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = connection.prepare("SELECT hash, path FROM staging")?;
    let file_entries = stmt.query_map([], |row| {
        Ok(FileEntry {
            hash: row.get(0)?,
            path: row.get(1)?,
        })
    })?;

    for file_entry in file_entries {
        let file_entry = file_entry.unwrap();
        println!("File Entry: {}:{}", file_entry.path, file_entry.hash)
    }

    Ok(())
}

pub fn staging_insert_batch(
    connection: &Connection,
    file_entries: Vec<FileEntry>,
) -> Result<(), Box<dyn Error>> {
    for file_entry in file_entries {
        connection.execute(
            "INSERT INTO staging (hash, path) VALUES (?1, ?2)",
            params![file_entry.hash, file_entry.path],
        )?;
    }
    Ok(())
}
