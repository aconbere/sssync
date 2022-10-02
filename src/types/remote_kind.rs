use std::fmt;

use clap::ArgEnum;
use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef,
};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, ArgEnum)]
pub enum RemoteKind {
    S3,
    Local,
}

impl RemoteKind {
    pub fn parse(s: &str) -> Result<RemoteKind, String> {
        match s {
            "S3" => Ok(RemoteKind::S3),
            "local" => Ok(RemoteKind::Local),
            _ => Err(format!("invalid kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match &self {
            RemoteKind::S3 => "S3",
            RemoteKind::Local => "local",
        }
    }
}

impl FromSql for RemoteKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match RemoteKind::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for RemoteKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

impl fmt::Display for RemoteKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemoteKind::S3 => write!(f, "S3"),
            RemoteKind::Local => write!(f, "local"),
        }
    }
}
