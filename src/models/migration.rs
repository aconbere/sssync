use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef,
};
use uuid::Uuid;

use crate::models::remote::Remote;
use crate::models::transfer::TransferKind;
use crate::types::remote_kind::RemoteKind;

#[derive(Debug)]
pub enum MigrationState {
    Waiting,
    Running,
    Complete,
    Canceled,
    Failed,
}

impl MigrationState {
    pub fn parse(s: &str) -> Result<MigrationState, String> {
        match s {
            "Waiting" => Ok(MigrationState::Waiting),
            "Running" => Ok(MigrationState::Running),
            "Complete" => Ok(MigrationState::Complete),
            "Canceled" => Ok(MigrationState::Canceled),
            "Failed" => Ok(MigrationState::Failed),
            _ => Err(format!("invalid Migration state kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            MigrationState::Waiting => "Waiting",
            MigrationState::Running => "Running",
            MigrationState::Complete => "Complete",
            MigrationState::Canceled => "Canceled",
            MigrationState::Failed => "Failed",
        }
    }
}

impl FromSql for MigrationState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match MigrationState::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for MigrationState {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

pub fn print_table(items: Vec<Migration>) {
    for i in items {
        print_migration_tabular(i)
    }
}

fn print_migration_tabular(m: Migration) {
    println!(
        "|{}, {:?}, {}, {}, {}, {:?}|",
        m.id, m.kind, m.remote_location, m.remote_kind, m.remote_name, m.state
    )
}

pub struct Migration {
    pub id: String,
    pub kind: TransferKind,
    pub remote_location: String,
    pub remote_kind: RemoteKind,
    pub remote_name: String,
    pub state: MigrationState,
}

impl Migration {
    pub fn new(kind: TransferKind, remote: &Remote) -> Self {
        Self {
            kind,
            id: Uuid::new_v4().hyphenated().to_string(),
            state: MigrationState::Waiting,
            remote_location: remote.location.to_string(),
            remote_kind: remote.kind,
            remote_name: remote.name.to_string(),
        }
    }
}
