use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub const STORE_DIR: &str = ".sssync";
pub const OBJECTS_DIR: &str = "objects";

pub fn has_store_dir(path: &Path) -> bool {
    path.join(STORE_DIR).exists()
}

pub fn store_path(root_path: &Path) -> PathBuf {
    root_path.join(STORE_DIR)
}

pub fn object_path(root_path: &Path, hash: String) -> PathBuf {
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

pub fn init(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.is_dir() {
        return Err(format!("path must be a directory: {}", path.display()).into());
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
    Ok(())
}
