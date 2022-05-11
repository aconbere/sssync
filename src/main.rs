use base64ct::{Base64, Encoding};
use sha2::{Digest, Sha256};
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;

struct FileEntry {
    path: String,
    hash: String,
}

fn hash_file(path: &Path) -> Result<String, Box<dyn Error>> {
    let mut hasher = Sha256::new();
    let mut file = fs::File::open(&path)?;
    io::copy(&mut file, &mut hasher)?;
    let hash = hasher.finalize();
    Ok(Base64::encode_string(&hash))
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
            println!("processing file: {}", path_str);
            let hash = hash_file(&path).unwrap();

            results.push(FileEntry {
                path: path_str.to_string(),
                hash: hash,
            });
        }
    }
    Ok(results)
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err(String::from("ssync requires two arguments"));
    }

    let source = Path::new(&args[1]);
    let destination = Path::new(&args[2]);

    if !source.is_dir() {
        return Err(format!("source {} must be a directory", source.display()));
    };

    if !destination.is_dir() {
        return Err(format!(
            "source {} must be a directory",
            destination.display()
        ));
    };

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
