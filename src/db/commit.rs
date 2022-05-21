use std::error::Error;

use rusqlite;
use rusqlite::params;
use rusqlite::Connection;

use crate::db::reference;
use crate::db::staging;
use crate::models::commit::Commit;

pub fn create_table(connection: &Connection) -> Result<(), Box<dyn Error>> {
    connection.execute(
        "
        CREATE TABLE
            commits (
                hash TEXT PRIMARY KEY,
                comment TEXT NOT NULL,
                author TEXT NOT NULL,
                created_unix_timestamp INTEGER NOT NULL,
                parent_hash TEXT
            )
        ",
        params![],
    )?;
    Ok(())
}

pub fn insert(connection: &Connection, commit: &Commit) -> Result<(), Box<dyn Error>> {
    println!("setting commit: {}", commit.hash);
    connection.execute(
        "
        INSERT INTO
            commits (hash, comment, author, created_unix_timestamp, parent_hash)
        VALUES
            (?1, ?2, ?3, ?4, ?5)
        ",
        params![
            commit.hash,
            commit.comment,
            commit.author,
            commit.created_unix_timestamp,
            commit.parent_hash,
        ],
    )?;
    println!("updating head");
    reference::update_head(connection, &commit.hash)?;
    println!("deleting staging");
    staging::delete(connection)?;
    Ok(())
}

pub fn get(connection: &Connection, hash: &str) -> Result<Commit, rusqlite::Error> {
    connection.query_row(
        "
        SELECT
            hash, comment, author, created_unix_timestamp, parent_hash
        FROM
            commits
        WHERE
            hash = ?1
        ",
        params![hash],
        |row| {
            Ok(Commit {
                hash: row.get(0)?,
                comment: row.get(1)?,
                author: row.get(2)?,
                created_unix_timestamp: row.get(3)?,
                parent_hash: row.get(4)?,
            })
        },
    )
}

pub fn get_all(connection: &Connection, hash: &str) -> Result<Vec<Commit>, rusqlite::Error> {
    let mut statement = connection.prepare(
        "
        WITH RECURSIVE
            log (hash, comment, author, created_unix_timestamp, parent_hash)
        AS (
            SELECT
                c.hash, c.comment, c.author, c.created_unix_timestamp, c.parent_hash
            FROM
                commits c
            WHERE
                c.hash = ?1

            UNION

            SELECT
                c.hash, c.comment, c.author, c.created_unix_timestamp, c.parent_hash
            FROM
                commits c, log l
            WHERE 
                c.hash = l.parent_hash
        )
        SELECT
            hash, comment, author, created_unix_timestamp, parent_hash
        FROM
            log
        ",
    )?;

    statement
        .query_map(params![hash], |row| {
            Ok(Commit {
                hash: row.get(0)?,
                comment: row.get(1)?,
                author: row.get(2)?,
                created_unix_timestamp: row.get(3)?,
                parent_hash: row.get(4)?,
            })
        })
        .into_iter()
        .flat_map(|e| e)
        .collect()
}

pub fn file_changes(
    connection: &Connection,
    left: &Vec<Commit>,
) -> Result<FileChanges, Box<dyn Error>> {
    // collect the trees for every commit in the vec
    // Concatentate the changes together into a set of
    // added and removed files
    Ok(())
}
