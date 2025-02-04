use anyhow::Result;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::meta::Meta;

pub fn create_table(connection: &Connection) -> Result<()> {
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

pub fn update(connection: &Connection, meta: &Meta) -> Result<()> {
    connection.execute(
        "
        UPDATE
            meta
        SET
           head = (?1)
        ",
        params![meta.head],
    )?;
    Ok(())
}

pub fn get(connection: &Connection) -> Result<Meta, rusqlite::Error> {
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
