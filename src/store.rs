use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::tree;

pub const STORE_DIR: &str = ".sssync";
pub const OBJECTS_DIR: &str = "objects";
pub const REMOTES_DIR: &str = "remotes";

pub fn has_store_dir(path: &Path) -> bool {
    path.join(STORE_DIR).exists()
}

pub fn store_path(root_path: &Path) -> PathBuf {
    root_path.join(STORE_DIR)
}

pub fn remote_db_path(root_path: &Path, name: &str) -> Result<PathBuf> {
    let path = store_path(root_path)
        .join(REMOTES_DIR)
        .join(format!("{}.db", name));

    if !path.exists() {
        return Err(anyhow!("Remote with name: {} does not exist!", name));
    }

    Ok(path)
}

pub fn db_path(root_path: &Path) -> PathBuf {
    root_path.join(".sssync/sssync.db")
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
) -> Result<()> {
    // Ensure the directory where we're going to write this file exists
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(object_path(root_path, hash), destination)?;
    Ok(())
}

pub fn exists(root_path: &Path, hash: &str) -> bool {
    let p = object_path(root_path, hash);
    p.exists()
}

// Writes the contents of the file found at source into the store
// with the hash hash.
pub fn insert_from(root_path: &Path, hash: &str, source: &Path) -> Result<()> {
    let p = object_path(root_path, hash);

    if !p.exists() {
        fs::copy(source, p)?;
    }
    Ok(())
}

pub fn init(path: &Path) -> Result<()> {
    if !path.is_dir() {
        return Err(
            anyhow!("path must be a directory: {}", path.display()).into()
        );
    }

    let store_path = store_path(path);

    if store_path.exists() {
        return Err(anyhow!(
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

pub fn apply_diff(path: &Path, diff: &tree::TreeDiff) -> Result<()> {
    for a in &diff.additions {
        let destination = path.join(&a.path);
        println!("copying: {} -> {}", a.file_hash, destination.display());
        export_to(path, &a.file_hash, &destination)?;
    }
    for a in &diff.changes {
        let destination = path.join(&a.path);
        println!("copying: {} -> {}", a.file_hash, destination.display());
        export_to(path, &a.file_hash, &destination)?;
    }
    for d in &diff.deletions {
        let destination = path.join(&d.path);
        println!("removing: {}", destination.display());
        // Skip errors with deletion, since they may not exist in the remote
        _ = fs::remove_file(destination);
    }
    Ok(())
}
