use anyhow::Result;
use rusqlite;
use rusqlite::params;
use rusqlite::Connection;
use rusqlite::OptionalExtension;

use crate::models::commit::Commit;

pub fn create_table(connection: &Connection) -> Result<()> {
    connection.execute(
        "
        CREATE TABLE
            commits (
                hash TEXT PRIMARY KEY,
                message TEXT NOT NULL,
                author TEXT NOT NULL,
                created_unix_timestamp INTEGER NOT NULL,
                parent_hash TEXT
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, commit: &Commit) -> Result<()> {
    connection.execute(
        "
        INSERT OR IGNORE INTO
            commits (
                hash,
                message,
                author,
                created_unix_timestamp,
                parent_hash
            )
        VALUES
            (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            commit.hash,
            commit.message,
            commit.author,
            commit.created_unix_timestamp,
            commit.parent_hash,
        ],
    )?;
    Ok(())
}

pub fn get(
    connection: &Connection,
    hash: &str,
) -> Result<Commit, rusqlite::Error> {
    connection.query_row(
        "
        SELECT
            hash, message, author, created_unix_timestamp, parent_hash
        FROM
            commits
        WHERE
            hash = ?1
        ",
        params![hash],
        |row| {
            Ok(Commit {
                hash: row.get(0)?,
                message: row.get(1)?,
                author: row.get(2)?,
                created_unix_timestamp: row.get(3)?,
                parent_hash: row.get(4)?,
            })
        },
    )
}

pub fn get_all(connection: &Connection) -> Result<Vec<Commit>> {
    let mut statement = connection.prepare(
        "
        SELECT
            hash, message, author, created_unix_timestamp, parent_hash
        FROM
            commits
        ",
    )?;

    let result: Vec<Commit> = statement
        .query_map(params![], |row| {
            Ok(Commit {
                hash: row.get(0)?,
                message: row.get(1)?,
                author: row.get(2)?,
                created_unix_timestamp: row.get(3)?,
                parent_hash: row.get(4)?,
            })
        })?
        .into_iter()
        .flatten()
        .collect();
    Ok(result)
}

pub fn get_children(
    connection: &Connection,
    head_hash: &str,
) -> Result<Vec<Commit>> {
    let mut statement = connection.prepare(
        "
        WITH RECURSIVE
            log (hash, message, author, created_unix_timestamp, parent_hash)
        AS (
            SELECT
                c.hash, c.message, c.author, c.created_unix_timestamp, c.parent_hash
            FROM
                commits c
            WHERE
                c.hash = ?1

            UNION

            SELECT
                c.hash, c.message, c.author, c.created_unix_timestamp, c.parent_hash
            FROM
                commits c, log l
            WHERE 
                c.hash = l.parent_hash
        )
        SELECT
            hash, message, author, created_unix_timestamp, parent_hash
        FROM
            log
        ",
    )?;

    let result: Vec<Commit> = statement
        .query_map(params![head_hash], |row| {
            Ok(Commit {
                hash: row.get(0)?,
                message: row.get(1)?,
                author: row.get(2)?,
                created_unix_timestamp: row.get(3)?,
                parent_hash: row.get(4)?,
            })
        })?
        .into_iter()
        .flatten()
        .collect();
    Ok(result)
}

pub fn get_by_ref_name(
    connection: &Connection,
    ref_name: &str,
) -> Result<Option<Commit>> {
    let result = connection
        .query_row(
            "
        SELECT
            c.hash, c.message, c.author, c.created_unix_timestamp, c.parent_hash
        FROM
            commits AS c
        JOIN
            refs
        ON
            refs.hash = c.hash
        WHERE
            refs.name = ?1
        ",
            params![ref_name],
            |row| {
                Ok(Commit {
                    hash: row.get(0)?,
                    message: row.get(1)?,
                    author: row.get(2)?,
                    created_unix_timestamp: row.get(3)?,
                    parent_hash: row.get(4)?,
                })
            },
        )
        .optional()?;

    Ok(result)
}
