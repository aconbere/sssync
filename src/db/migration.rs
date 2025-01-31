use anyhow::Result;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::migration::{Migration, MigrationState};

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE
            migrations (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                remote_name TEXT NOT NULL,
                remote_kind TEXT NOT NULL,
                remote_location TEXT NOT NULL,
                state TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, migration: &Migration) -> Result<()> {
    connection.execute(
        "
        INSERT INTO
            migrations (
                id,
                kind,
                remote_name,
                remote_kind,
                remote_location,
                state
            )
        VALUES
            (?1, ?2, ?3, ?4, ?5, ?6)
        ",
        params![
            migration.id,
            migration.kind,
            migration.remote_name,
            migration.remote_kind,
            migration.remote_location,
            migration.state
        ],
    )?;
    Ok(())
}

pub fn get_all(
    connection: &Connection,
) -> Result<Vec<Migration>, rusqlite::Error> {
    let mut stmt = connection.prepare(
        "
        SELECT
            id,
            kind,
            remote_name,
            remote_kind,
            remote_location,
            state
        FROM
            migrations
        ",
    )?;

    stmt.query_map(params![], |row| {
        Ok(Migration {
            id: row.get(0)?,
            kind: row.get(1)?,
            remote_name: row.get(2)?,
            remote_kind: row.get(3)?,
            remote_location: row.get(4)?,
            state: row.get(5)?,
        })
    })
    .into_iter()
    .flatten()
    .collect()
}

//pub fn get(
//    connection: &Connection,
//    id: &str,
//) -> Result<Migration, rusqlite::Error> {
//    connection.query_row(
//        "
//        SELECT
//            id,
//            kind,
//            remote_name,
//            remote_kind,
//            remote_location,
//            state
//        FROM
//            migrations
//        WHERE
//            id = ?1
//        ",
//        params![id],
//        |row| {
//            Ok(Migration {
//                id: row.get(0)?,
//                kind: row.get(1)?,
//                remote_name: row.get(2)?,
//                remote_kind: row.get(3)?,
//                remote_location: row.get(4)?,
//                state: row.get(5)?,
//            })
//        },
//    )
//}

pub fn set_state(
    connection: &Connection,
    migration: &Migration,
    state: MigrationState,
) -> Result<()> {
    connection.execute(
        "
        UPDATE
            migrations
        SET
            state = ?2
        WHERE
            id = ?1
        ",
        params![migration.id, state],
    )?;
    Ok(())
}
