use std::error::Error;

use crate::types::remote_kind::RemoteKind;

pub struct Remote {
    pub name: String,
    pub kind: RemoteKind,
    pub location: String,
}

impl Remote {
    pub fn new(name: &str, kind: RemoteKind, location: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: name.to_string(),
            kind: kind,
            location: location.to_string(),
        })
    }
}
