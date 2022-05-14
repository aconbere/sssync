use clap::{Parser, Subcommand};
use std::error::Error;
use std::fs;
use std::path::Path;

use crate::actions::{add, checkout, commit, init, log, remote, reset, status, tree};
use crate::db::get_connection;
use crate::store::get_root_path;

#[derive(Subcommand, Debug)]
pub enum Remote {
    Add {
        path: String,
        name: String,
        url: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Checkout {
        path: String,
        hash: String,
    },
    Reset {
        path: String,
    },
    Log {
        path: String,
    },
    Commit {
        path: String,
    },
    Status {
        path: String,
    },
    Init {
        path: String,
    },
    Add {
        path: String,
    },
    Tree {
        path: String,
        hash: String,
    },

    Remote {
        #[clap(subcommand)]
        action: Remote,
    },
    //Fetch { path: String, remote: String },
    //Push { path: String, remote: String },
    //Diff { path: String, remote: String },
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
        Action::Remote { action } => {
            match action {
                Remote::Add { path, name, url } => {
                    let path = fs::canonicalize(path)?;
                    let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
                    let connection = get_connection(root_path)?;
                    remote::add(&connection, name, url)
                }
            }
        }
        Action::Commit { path } => {
            println!("Action::Commit: {}", path);
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let connection = get_connection(root_path)?;
            commit::commit(&connection, &path)
        }
        Action::Status { path } => {
            println!("Action::Status: {}", path);
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let connection = get_connection(root_path)?;
            status::status(&connection, &path)
        }
        Action::Init { path } => {
            println!("Action::Init: {}", path);
            let path = Path::new(path);
            if !path.is_dir() {
                return Err(format!("desintation {} must be a directory", path.display()).into());
            }
            init::init(path)
        }
        Action::Add { path } => {
            println!("Action::Add: {}", path);
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let rel_path = path.strip_prefix(root_path)?;
            let connection = get_connection(root_path)?;
            add::add(&connection, &path, &root_path, &rel_path)
        }
        Action::Log { path } => {
            println!("Action::Log: {}", path);
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let connection = get_connection(root_path)?;
            log::log(&connection, &path)
        }
        Action::Checkout { path, hash } => {
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let connection = get_connection(root_path)?;
            checkout::checkout(&connection, hash)
        }
        Action::Reset { path } => {
            println!("Action::Reset: {}", path);
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let connection = get_connection(root_path)?;
            reset::reset(&connection, &path)
        }
        Action::Tree { path, hash } => {
            println!("Action::Reset: {}", path);
            let path = fs::canonicalize(path)?;
            let root_path = get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;
            let connection = get_connection(root_path)?;
            tree::tree(&connection, hash)
        }
        //Action::Fetch { remote } => Ok(()),
        //Action::Push { remote } => Ok(()),
        //Action::Diff { remote } => Ok(()),
    }
}
