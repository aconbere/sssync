use std::path::Path;

use anyhow::Result;
use url::{ParseError, Url};

use crate::types::remote_kind::RemoteKind;

pub fn remote_object_path(url: &str, hash: &str) -> Result<Url, ParseError> {
    let u = Url::parse(url)?;

    let remote_directory = Path::new(u.path());
    let p = remote_directory.join(".sssync/objects").join(hash);
    let new_url = format!(
        "{scheme}://{host_str}{path}",
        scheme = u.scheme(),
        host_str = u.host_str().unwrap_or(""),
        path = p.to_str().unwrap(),
    );
    Url::parse(&new_url)
}

pub struct Remote {
    pub name: String,
    pub kind: RemoteKind,
    pub location: String,
}

impl Remote {
    pub fn new(name: &str, kind: RemoteKind, location: &str) -> Result<Self> {
        Ok(Self {
            kind,
            name: name.to_string(),
            location: location.to_string(),
        })
    }
}
