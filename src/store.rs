use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub const STORE_DIR: &str = ".sssync";
pub const OBJECTS_DIR: &str = "objects";
pub const REMOTES_DIR: &str = "remotes";

pub fn has_store_dir(path: &Path) -> bool {
    path.join(STORE_DIR).exists()
}

pub fn store_path(root_path: &Path) -> PathBuf {
    root_path.join(STORE_DIR)
}

pub fn object_path(root_path: &Path, hash: &str) -> PathBuf {
    let mut p = PathBuf::new();
    p.push(STORE_DIR);
    p.push("objects");
    p.push(hash);
    root_path.join(p)
}

pub fn get_root_path(path: &Path) -> Option<&Path> {
    if has_store_dir(path) {
        Some(path)
    } else {
        match path.parent() {
            Some(parent) => get_root_path(parent),
            None => None,
        }
    }
}

// Writes the contents of the store indexed by hash to the file
// at the path destination.
pub fn export_to(
    root_path: &Path,
    hash: &str,
    destination: &Path,
) -> Result<(), Box<dyn Error>> {
    let p = object_path(root_path, hash);
    fs::copy(p, destination)?;
    Ok(())
}

// Writes the contents of the file found at source into the store
// with the hash hash.
pub fn insert_from(
    root_path: &Path,
    hash: &str,
    source: &Path,
) -> Result<(), Box<dyn Error>> {
    let p = object_path(root_path, hash);

    if !p.exists() {
        fs::copy(source, p)?;
    }
    Ok(())
}

pub fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.is_dir() {
        return Err(
            format!("path must be a directory: {}", path.display()).into()
        );
    }

    let store_path = store_path(path);

    if store_path.exists() {
        return Err(format!(
            "path {} already contains a {} directory",
            path.display(),
            STORE_DIR
        )
        .into());
    }

    fs::create_dir(&store_path)?;
    fs::create_dir(&store_path.join(OBJECTS_DIR))?;
    fs::create_dir(&store_path.join(REMOTES_DIR))?;
    Ok(())
}
