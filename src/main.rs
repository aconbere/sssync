use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

use base64ct::{Base64, Encoding};
use rusqlite::params;
use rusqlite::Connection;
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

struct FileEntry {
    path: String,
    hash: String,
}

fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut hasher = Xxh3Writer::new();

    let mut file = fs::File::open(&path)?;
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.hasher.digest128();
    Ok(Base64::encode_string(&u128_to_byte_array(hash)))
}
fn all_files(start: &Path) -> Result<Vec<FileEntry>, Box<dyn Error>> {
    all_files_inner(start, PathBuf::from("./"))
}

fn all_files_inner(start: &Path, up_to_path: PathBuf) -> Result<Vec<FileEntry>, Box<dyn Error>> {
    let contents = fs::read_dir(start)?;
    let mut results: Vec<FileEntry> = Vec::new();

    for entry in contents {
        let entry = entry?;
        let path = entry.path();
        let mut next_up_to_path = up_to_path.clone();
        next_up_to_path.push(entry.file_name());

        if path.is_dir() {
            let sub_results = all_files_inner(&path, next_up_to_path)?;
            results.extend(sub_results);
        } else {
            let path_str = next_up_to_path.to_str().unwrap();
            let hash = hash_file(&path).unwrap();
            println!("processing file: {} : {}", path_str, hash);

            results.push(FileEntry {
                path: path_str.to_string(),
                hash: hash,
            });
        }
    }
    Ok(results)
}

fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut db_path = path.to_path_buf();
    db_path.push(".sssync.db");
    let connection = Connection::open(db_path.as_path())?;
    connection.execute(
        "CREATE TABLE objects (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err(String::from("ssync requires two arguments").into());
    }

    let source = Path::new(&args[1]);
    let destination = Path::new(&args[2]);

    if !source.is_dir() {
        return Err(format!("source {} must be a directory", source.display()).into());
    };

    if !destination.is_dir() {
        return Err(format!("source {} must be a directory", destination.display()).into());
    };

    init(source)?;

    println!(
        "syncing {} into {}",
        source.display(),
        destination.display()
    );

    let files = all_files(source).unwrap_or(vec![]);

    for f in files {
        println!("found file: {} with hash: {}", f.path, f.hash);
    }
    Ok(())
}

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
