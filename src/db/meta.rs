use std::error::Error;

use rusqlite::params;
use rusqlite::Connection;

use crate::models::meta::Meta;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    println!("meta::create_table");
    connection.execute(
        "
        CREATE TABLE
            meta (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                head TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn update(connection: &Connection, meta: &Meta) -> Result<(), Box<dyn Error>> {
    println!("meta::update");
    connection.execute(
        "
        INSERT INTO
            meta (head)
        VALUES
            (?1)
        ",
        params![meta.head],
    )?;
    Ok(())
}

pub fn get(connection: &Connection) -> Result<Meta, rusqlite::Error> {
    println!("meta::get");
    connection.query_row(
        "
        SELECT
            head
        FROM
            meta
        ORDER BY
            id DESC
        LIMIT
            1
        ",
        params![],
        |row| Ok(Meta { head: row.get(0)? }),
    )
}
