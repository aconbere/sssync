use std::env;
use std::fs;
use std::io;
use std::path::Path;
use std::process::exit;
use sha2::{Sha256, Digest};

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
        return Err(format!("source {} must be a directory", destination.display()));
    };

    match fs::read_dir(source) {
        Ok(contents) => {
            for entry in contents {
                match entry {
                    Ok(entry) => {
                        let path = entry.path();
                        if path.is_dir() {
                            // recurse
                        } else {
                            let mut hasher = Sha256::new();
                            let mut file = match fs::File::open(&path) {
                                Ok(f) => f,
                                Err(_) => {
                                    return Err(format!("Failed to open file: {}", path.display())); 
                                }
                            };

                            match io::copy(&mut file, &mut hasher) {
                                Ok(_) => {},
                                Err(_) => {
                                    return Err(format!("Failed to copy bytes from file, {}", path.display())); 
                                }
                            };
                            let hash_bytes = hasher.finalize();
                        }

                    }
                    Err(_) => {}
                }
            }
        }
        Err(_) => {
            return Err(format!("could not read source directory: {}", source.display()));
        }
    }


    println!("syncing {} into {}", source.display(), destination.display());
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
