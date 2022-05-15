use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::remote::Remote;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            remotes (
                name PRIMARY KEY,
                url NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, remote: &Remote) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        INSERT INTO remotes (name, url)
        VALUES (?1, ?2)
        ",
        params![remote.name, String::from(remote.url.clone())],
    )?;
    Ok(())
}

struct IntermediateRemote {
    name: String,
    url: String,
}

fn get_intermediate(connection: &Connection) -> Result<Vec<IntermediateRemote>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            name, url
        FROM
            remotes
        ",
    )?;

    statement
        .query_map(params![], |row| {
            Ok(IntermediateRemote {
                name: row.get(0)?,
                url: row.get(1)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}

pub fn get_all(connection: &Connection) -> Result<Vec<Remote>, Box<dyn Error>> {
    let inter = get_intermediate(connection)?;
    inter.iter().map(|e| Remote::new(&e.name, &e.url)).collect()
}
