use clap::{Parser, Subcommand};
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::actions::{add, init, status};
use crate::db::get_connection;

#[derive(Subcommand, Debug)]
pub enum Action {
    Commit,
    Status { path: String },
    Init { path: String },
    Add { path: String },
    Fetch { remote: String },
    Push { remote: String },
    Diff { remote: String },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,
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

pub fn run() -> Result<(), Box<dyn Error>> {
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
