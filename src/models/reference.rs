use rusqlite::types::{
    FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef,
};

pub static LOCAL: &str = "local";

/* A reference is a name attached to a commit.
 *
 * Right now the only supported kind of reference is a branch.
 */
#[derive(Debug)]
pub struct Reference {
    pub name: String,
    #[allow(dead_code)]
    pub kind: Kind,
    pub hash: String,
    #[allow(dead_code)]
    pub remote: Option<String>,
}

impl Reference {
    #[allow(dead_code)]
    pub fn new(
        name: &str,
        kind: Kind,
        hash: &str,
        remote: Option<&str>,
    ) -> Self {
        Self {
            kind,
            name: name.to_string(),
            hash: hash.to_string(),
            remote: remote.map(|s| s.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum Kind {
    Branch,
}

impl Kind {
    pub fn parse(s: &str) -> Result<Kind, String> {
        match s {
            "branch" => Ok(Kind::Branch),
            _ => Err(format!("invalid reference kind: {}", s)),
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
