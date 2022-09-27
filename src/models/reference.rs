use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

pub enum Kind {
    Branch,
}

impl Kind {
    pub fn parse(s: &str) -> Result<Kind, String> {
        match s {
            "branch" => Ok(Kind::Branch),
            _ => Err(format!("invalid kind: {}", s)),
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Kind::Branch => "branch",
        }
    }
}

impl FromSql for Kind {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        value.as_str().and_then(|s| match Kind::parse(s) {
            Ok(rk) => Ok(rk),
            Err(_) => Err(FromSqlError::InvalidType),
        })
    }
}

impl ToSql for Kind {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_str()))
    }
}

pub struct Reference {
    pub name: String,
    pub kind: Kind,
    pub hash: String,
}

impl Reference {
    #[allow(dead_code)]
    pub fn new(name: &str, kind: Kind, hash: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: kind,
            hash: hash.to_string(),
        }
    }
}
