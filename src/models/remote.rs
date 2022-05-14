use std::error::Error;

use url::Url;

pub struct Remote {
    pub name: String,
    pub url: Url,
}

impl Remote {
    pub fn new(name: &str, url: &str) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            name: name.to_string(),
            url: Url::parse(url)?,
        })
    }
}
