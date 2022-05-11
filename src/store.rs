use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub const STORE_DIR: &str = ".sssync";
pub const OBJECTS_DIR: &str = "objects";

pub fn has_store_dir(path: &Path) -> bool {
    path.join(STORE_DIR).exists()
}

pub fn store_path(p: &Path) -> PathBuf {
    p.join(STORE_DIR)
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
