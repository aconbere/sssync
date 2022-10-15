use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef,
};

#[derive(Debug, Clone)]
pub enum TransferKind {
    Upload,
    Download,
}

impl TransferKind {
    pub fn parse(s: &str) -> Result<TransferKind, String> {
        match s {
            "Upload" => Ok(TransferKind::Upload),
            "Download" => Ok(TransferKind::Download),
            _ => Err(format!("invalid kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            TransferKind::Upload => "Upload",
            TransferKind::Download => "Download",
        }
    }
}

impl FromSql for TransferKind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match TransferKind::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for TransferKind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

#[derive(Debug)]
pub enum TransferState {
    Waiting,
    Running,
    Failed,
    Complete,
}

impl TransferState {
    pub fn parse(s: &str) -> Result<TransferState, String> {
        match s {
            "Waiting" => Ok(TransferState::Waiting),
            "Running" => Ok(TransferState::Running),
            "Failed" => Ok(TransferState::Failed),
            "Complete" => Ok(TransferState::Complete),
            _ => Err(format!("invalid kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            TransferState::Waiting => "Waiting",
            TransferState::Running => "Running",
            TransferState::Failed => "Failed",
            TransferState::Complete => "Complete",
        }
    }
}

impl FromSql for TransferState {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match TransferState::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for TransferState {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

pub fn print_table(uploads: Vec<Transfer>) {
    for u in uploads {
        println!("{}, {:?}", u.object_hash, u.state)
    }
}

pub struct Transfer {
    pub migration_id: String,
    pub object_hash: String,
    pub state: TransferState,
    pub kind: TransferKind,
}

impl Transfer {
    pub fn new(
        migration_id: &str,
        object_hash: &str,
        kind: TransferKind,
    ) -> Self {
        Self {
            migration_id: migration_id.to_string(),
            object_hash: object_hash.to_string(),
            state: TransferState::Waiting,
            kind,
        }
    }
}
