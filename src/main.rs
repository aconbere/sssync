use base64ct::{Base64, Encoding};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::exit;

struct FileEntry {
    path: String,
    hash: String,
}

fn hash_file(path: &Path) -> Result<String, String> {
    let mut hasher = Sha256::new();

    let mut file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => {
            return Err(format!("Failed to open file: {}", path.display()));
        }
    };

    match io::copy(&mut file, &mut hasher) {
        Ok(_) => {}
        Err(_) => {
            return Err(format!(
                "Failed to copy bytes from file, {}",
                path.display()
            ));
        }
    };
    let hash = hasher.finalize();
    Ok(Base64::encode_string(&hash))
}
fn all_files(start: &Path) -> Result<Vec<FileEntry>, String> {
    all_files_inner(start, PathBuf::from("./"))
}

fn all_files_inner(start: &Path, up_to_path: PathBuf) -> Result<Vec<FileEntry>, String> {
    match fs::read_dir(start) {
        Ok(contents) => {
            let mut results = Vec::new();

            for entry in contents {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        let mut next_up_to_path = up_to_path.clone();
                        next_up_to_path.push(entry.file_name());

                        if path.is_dir() {
                            match all_files_inner(&path, next_up_to_path) {
                                Ok(sub_results) => {
                                    results.extend(sub_results);
                                }
                                Err(err) => {
                                    return Err(err);
                                }
                            };
                        } else {
                            let hash = hash_file(&path).unwrap();

                            let path_str = match next_up_to_path.to_str() {
                                Some(s) => s,
                                None => {
                                    return Err(format!(
                                        "path contained invalid unicode, {}",
                                        path.display()
                                    ));
                                }
                            };

                            results.push(FileEntry {
                                path: path_str.to_string(),
                                hash: hash,
                            });
                        }
                    }
                    Err(_) => {
                        return Err(String::from("whatever"));
                    }
                };
            }
            return Ok(results);
        }
        Err(_) => {
            return Err(format!(
                "could not read source directory: {}",
                start.display()
            ));
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
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
