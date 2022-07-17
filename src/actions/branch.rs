use std::error::Error;

use rusqlite::Connection;

pub fn add(connection: &Connection, name: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn switch(connection: &Connection, name: &str) -> Result<(), Box<dyn Error>> {
    Ok(())
}

pub fn list(connection: &Connection) -> Result<(), Box<dyn Error>> {
    Ok(())
}
