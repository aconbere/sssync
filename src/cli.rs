use clap::{Parser, Subcommand};
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::actions::{add, commit, init, status};
use crate::db::get_connection;
use crate::store::get_root_path;

#[derive(Subcommand, Debug)]
pub enum Action {
    Commit { path: String },
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

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.action {
        Action::Commit { path } => {
            let path = fs::canonicalize(path)?;
            match get_root_path(&path) {
                Some(root_path) => {
                    let connection = get_connection(root_path)?;
                    commit(&connection, &path)
                }
                None => {
                    return Err(format!("not in a sssync'd directory: {}", path.display()).into())
                }
            }
        }
        Action::Status { path } => {
            let path = fs::canonicalize(path)?;
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
            println!("Action::Add: {}", path);
            let path = fs::canonicalize(path)?;
            match get_root_path(&path) {
                Some(root_path) => {
                    println!("Root Path: {}", root_path.display());
                    let rel_path = path.strip_prefix(root_path)?;
                    let connection = get_connection(root_path)?;
                    add(&connection, &path, &root_path, &rel_path)
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
