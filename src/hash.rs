use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

use base64ct::{Base64, Encoding};
use xxhash_rust::xxh3::Xxh3;

fn u128_to_byte_array(n: u128) -> [u8; 16] {
    let mut out: [u8; 16] = [0; 16];

    for i in 0..16 {
        out[i] = (n >> (16 - i)) as u8;
    }

    out
}

struct Xxh3Writer {
    hasher: Xxh3,
}

impl Xxh3Writer {
    pub const fn new() -> Self {
        Self {
            hasher: Xxh3::new(),
        }
    }
}

impl Write for Xxh3Writer {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        self.hasher.update(buf);
        return Ok(buf.len());
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        return Ok(());
    }
}

pub fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut hasher = Xxh3Writer::new();
    let mut file = File::open(&path)?;
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.hasher.digest128();
    Ok(Base64::encode_string(&u128_to_byte_array(hash)))
}
