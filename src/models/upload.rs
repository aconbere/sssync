use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

// Note:: Rename to Transfer

pub enum UploadState {
    Waiting,
    Running,
    Failed,
    Complete,
}

impl UploadState {
    pub fn parse(s: &str) -> Result<UploadState, String> {
        match s {
            "Waiting" => Ok(UploadState::Waiting),
            "Running" => Ok(UploadState::Running),
            "Failed" => Ok(UploadState::Failed),
            "Complete" => Ok(UploadState::Complete),
            _ => Err(format!("invalid kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            UploadState::Waiting => "Waiting",
            UploadState::Running => "Running",
            UploadState::Failed => "Failed",
            UploadState::Complete => "Complete",
        }
    }
}

impl FromSql for UploadState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match UploadState::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for UploadState {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

pub struct Upload {
    pub migration_id: String,
    pub object_hash: String,
    pub state: UploadState,
}

impl Upload {
    pub fn new(migration_id: &str, object_hash: &str) -> Self {
        Self {
            migration_id: migration_id.to_string(),
            object_hash: object_hash.to_string(),
            state: UploadState::Waiting,
        }
    }
}
