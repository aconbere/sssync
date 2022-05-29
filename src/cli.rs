use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use rusqlite::Connection;
use tokio;

use crate::actions::{add, checkout, commit, init, log, remote, reset, status, tree};
use crate::db::repo_db_path;
use crate::store::get_root_path;
use crate::types::remote_kind::RemoteKind;

#[derive(Subcommand, Debug)]
pub enum Remote {
    Add {
        name: String,

        #[clap(long, arg_enum)]
        kind: RemoteKind,

        #[clap(long)]
        location: String,
    },
    List,
    Init {
        name: String,
    },
    Push {
        name: String,
    },
    Remove {
        name: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum Action {
    Checkout {
        hash: String,
    },
    Reset,
    Log,
    Commit,
    Status,
    Init,
    Add,
    Tree {
        hash: String,
    },

    Remote {
        #[clap(subcommand)]
        action: Remote,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    path: Option<PathBuf>,
    #[clap(subcommand)]
    action: Action,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let path = cli
        .path
        .as_deref()
        .unwrap_or(Path::new("."))
        .canonicalize()?;

    if let Action::Init = &cli.action {
        println!("Action::Init: {}", path.display());
        if !path.is_dir() {
            return Err(format!("desintation {} must be a directory", path.display()).into());
        }
        init::init(&path)?;
        return Ok(());
    }

    let root_path =
        get_root_path(&path).ok_or(format!("not in a sssync'd directory: {}", path.display()))?;

    let connection = Connection::open(repo_db_path(&root_path))?;

    match &cli.action {
        Action::Remote { action } => match action {
            Remote::Add {
                name,
                kind,
                location,
            } => {
                println!("Remote::Add: {} {} {}", name, kind, location);
                remote::add(&connection, name, kind, location)
            }
            Remote::List => {
                println!("Remote::List");
                remote::list(&connection)
            }
            Remote::Init { name } => {
                println!("Remote::Init");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::init(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::Push { name } => {
                println!("Remote::Push");
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::push(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::Remove { name } => {
                println!("Remote::Remove");
                println!("Removing remote {}", name);
                remote::remove(&connection, name)?;
                Ok(())
            }
        },
        Action::Commit => {
            println!("Action::Commit");
            commit::commit(&connection, &root_path)
        }
        Action::Status => {
            println!("Action::Status");
            status::status(&connection, &path)
        }
        Action::Init => {
            // handled above since it doesn't have a root_path yet
            Ok(())
        }
        Action::Add => {
            println!("Action::Add");
            let rel_path = path.strip_prefix(root_path)?;
            add::add(&connection, &path, &root_path, &rel_path)
        }
        Action::Log => {
            println!("Action::Log");
            log::log(&connection)
        }
        Action::Checkout { hash } => checkout::checkout(&connection, hash),
        Action::Reset => {
            println!("Action::Reset");
            reset::reset(&connection, &path)
        }
        Action::Tree { hash } => {
            println!("Action::Tree");
            tree::tree(&connection, hash)
        }
    }
}
