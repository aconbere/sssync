use std::error::Error;
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

use base64ct::{Base64, Encoding};
use clap::{Parser, Subcommand};
use rusqlite::params;
use rusqlite::Connection;
use xxhash_rust::xxh3::Xxh3;

#[derive(Subcommand, Debug)]
enum Action {
    Commit,
    Status { path: String },
    Init { path: String },
    Add { path: String },
    Fetch { remote: String },
    Push { remote: String },
    Diff { remote: String },
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    action: Action,
}

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

impl FileEntry {
    pub fn hash(full_path: &Path, relative_path: &Path) -> Result<Self, Box<dyn Error>> {
        match hash_file(full_path) {
            Ok(hash) => match relative_path.to_str() {
                Some(relative_path_str) => Ok(Self {
                    path: relative_path_str.to_string(),
                    hash: hash,
                }),
                None => Err(format!("Invalid path: {}", relative_path.display()).into()),
            },
            Err(e) => Err(e.into()),
        }
    }
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
        let mut relative_path = up_to_path.clone();
        relative_path.push(entry.file_name());

        if path.is_dir() {
            let sub_results = all_files_inner(&path, relative_path)?;
            results.extend(sub_results);
        } else {
            let file_entry = FileEntry::hash(path.as_path(), relative_path.as_path())?;
            println!("processing file: {} : {}", file_entry.path, file_entry.hash);
            results.push(file_entry);
        }
    }
    Ok(results)
}

fn get_connection(path: &Path) -> Result<Connection, Box<dyn Error>> {
    let mut db_path = path.to_path_buf();
    db_path.push(".sssync.db");
    let connection = Connection::open(db_path.as_path())?;
    Ok(connection)
}

fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    let connection = get_connection(path)?;
    connection.execute(
        "CREATE TABLE objects (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;

    connection.execute(
        "CREATE TABLE staging (hash TEXT primary key, path TEXT not null)",
        params![],
    )?;

    connection.execute("CREATE TABLE commits (hash TEXT primary key)", params![])?;
    Ok(())
}

fn has_db_file(path: &Path) -> bool {
    path.join(".sssync.db").exists()
}

fn get_root_path(path: &Path) -> Option<&Path> {
    if has_db_file(path) {
        Some(path)
    } else {
        match path.parent() {
            Some(parent) => get_root_path(parent),
            None => None,
        }
    }
}

fn stage_files(
    connection: &Connection,
    file_entries: Vec<FileEntry>,
) -> Result<(), Box<dyn Error>> {
    for file_entry in file_entries {
        connection.execute(
            "INSERT INTO staging (hash, path) VALUES (?1, ?2)",
            params![file_entry.hash, file_entry.path],
        )?;
    }
    Ok(())
}

fn add(connection: &Connection, path: &Path) -> Result<(), Box<dyn Error>> {
    if path.is_dir() {
        let file_entries = all_files(path).unwrap_or(vec![]);
        stage_files(connection, file_entries)?;
        return Ok(());
    }

    if path.is_file() {
        let file_entry = FileEntry::hash(path, path)?;
        println!("processing file: {} : {}", file_entry.path, file_entry.hash);
        stage_files(connection, vec![file_entry])?;
        return Ok(());
    }

    Ok(())
}

fn select_staging_info(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let mut stmt = connection.prepare("SELECT hash, path FROM staging")?;
    let file_entries = stmt.query_map([], |row| {
        Ok(FileEntry {
            hash: row.get(0)?,
            path: row.get(1)?,
        })
    })?;

    for file_entry in file_entries {
        let file_entry = file_entry.unwrap();
        println!("File Entry: {}:{}", file_entry.path, file_entry.hash)
    }

    Ok(())
}

fn status(connection: &Connection, _path: &Path) -> Result<(), Box<dyn Error>> {
    select_staging_info(connection)?;
    Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.action {
        Action::Commit => Ok(()),
        Action::Status { path } => {
            let path = fs::canonicalize(path)?;
            // struggling to get errors to type correctly here
            //let root_path = get_root_path(path)
            //    .ok_or(format!("No sssync directory found {}", path.display()).into())?;
            match get_root_path(&path) {
                Some(root_path) => {
                    let connection = get_connection(root_path)?;
                    status(&connection, &path)
                }
                None => {
                    return Err(format!("not in a sssync'd directory: {}", path.display()).into())
                }
            }
        }
        Action::Init { path } => {
            let path = Path::new(path);
            if !path.is_dir() {
                return Err(format!("desintation {} must be a directory", path.display()).into());
            }
            init(path)
        }
        Action::Add { path } => {
            let path = fs::canonicalize(path)?;
            // struggling to get errors to type correctly here
            //let root_path = get_root_path(path)
            //    .ok_or(format!("No sssync directory found {}", path.display()).into())?;
            match get_root_path(&path) {
                Some(root_path) => {
                    let connection = get_connection(root_path)?;
                    add(&connection, &path)
                }
                None => {
                    return Err(format!("not in a sssync'd directory: {}", path.display()).into())
                }
            }
        }
        Action::Fetch { remote } => Ok(()),
        Action::Push { remote } => Ok(()),
        Action::Diff { remote } => Ok(()),
    }
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
