use anyhow::Result;
use rusqlite::params;
use rusqlite::Connection;

use crate::models::transfer::{Transfer, TransferState};

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE
            transfers (
                migration_id TEXT NOT NULL,
                object_hash TEXT NOT NULL,
                state TEXT NOT NULL,
                kind TEXT NOT NULL
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, transfer: &Transfer) -> Result<()> {
    connection.execute(
        "
        INSERT INTO
            transfers (
                migration_id,
                object_hash,
                state,
                kind
            )
        VALUES
            (?1, ?2, ?3, ?4)
        ",
        params![
            transfer.migration_id,
            transfer.object_hash,
            transfer.state,
            transfer.kind
        ],
    )?;
    Ok(())
}

pub fn get_all(
    connection: &Connection,
    migration_id: &str,
) -> Result<Vec<Transfer>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            migration_id, object_hash, state, kind
        FROM
            transfers
        WHERE
            migration_id = ?1
        ",
    )?;
    statement
        .query_map(params![migration_id], |row| {
            Ok(Transfer {
                migration_id: row.get(0)?,
                object_hash: row.get(1)?,
                state: row.get(2)?,
                kind: row.get(3)?,
            })
        })
        .into_iter()
        .flatten()
        .collect()
}

pub fn get_all_with_state(
    connection: &Connection,
    migration_id: &str,
    state: TransferState,
) -> Result<Vec<Transfer>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        SELECT
            migration_id, object_hash, state, kind
        FROM
            transfers
        WHERE
            migration_id = ?1 AND
            state = ?2
        ",
    )?;
    statement
        .query_map(params![migration_id, state], |row| {
            Ok(Transfer {
                migration_id: row.get(0)?,
                object_hash: row.get(1)?,
                state: row.get(2)?,
                kind: row.get(3)?,
            })
        })
        .into_iter()
        .flatten()
        .collect()
}

pub fn get_waiting_for_migration(
    connection: &Connection,
    migration_id: &str,
) -> Result<Vec<Transfer>, rusqlite::Error> {
    get_all_with_state(connection, migration_id, TransferState::Waiting)
}

pub fn set_state(
    connection: &Connection,
    transfer: &Transfer,
    state: TransferState,
) -> Result<()> {
    connection.execute(
        "
        UPDATE
            transfers
        SET
            state = ?3
        WHERE
            migration_id = ?1 AND
            object_hash = ?2
        ",
        params![transfer.migration_id, transfer.object_hash, state],
    )?;
    Ok(())
}
