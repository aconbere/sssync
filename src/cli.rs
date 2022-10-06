use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use rusqlite::Connection;
use tokio;

use crate::actions::{
    add, branch, checkout, commit, init, log, remote, reset, status, tree,
};
use crate::db::repo_db_path;
use crate::store::get_root_path;
use crate::types::remote_kind::RemoteKind;

#[derive(Subcommand, Debug)]
pub enum Branch {
    Add { name: String },
    Switch { name: String },
    List,
}

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
    Fetch {
        name: String,
    },
    Sync {
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
    Init {
        path: PathBuf,
    },
    Add {
        path: PathBuf,
    },
    Tree {
        hash: String,
    },
    Remote {
        #[clap(subcommand)]
        action: Remote,
    },
    Branch {
        #[clap(subcommand)]
        action: Branch,
    },
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let pwd = Path::new(".").canonicalize()?;
    let root_path = get_root_path(&pwd)
        .ok_or(format!("not in a sssync'd directory: {}", pwd.display()))?;

    let connection = Connection::open(repo_db_path(&root_path))?;

    match &cli.action {
        Action::Remote { action } => match action {
            Remote::Add {
                name,
                kind,
                location,
            } => remote::add(&connection, name, kind, location),
            Remote::List => remote::list(&connection),
            Remote::Init { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::init(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::Push { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::push(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::Sync { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::sync(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::Fetch { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::fetch(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::Remove { name } => {
                remote::remove(&connection, name)?;
                Ok(())
            }
        },
        Action::Branch { action } => match action {
            Branch::Add { name } => branch::add(&connection, name, None),
            Branch::Switch { name } => {
                branch::switch(&connection, name, &root_path)
            }
            Branch::List => branch::list(&connection),
        },
        Action::Commit => commit::commit(&connection, &root_path),
        Action::Status => {
            status::status(&connection, &root_path)?;
            Ok(())
        }
        Action::Init { path } => {
            if !path.is_dir() {
                return Err(format!(
                    "desintation {} must be a directory",
                    path.display()
                )
                .into());
            }
            init::init(&path)?;
            Ok(())
        }
        Action::Add { path } => {
            let cp = path.canonicalize()?;
            let rel_path = cp.strip_prefix(root_path)?;
            add::add(&connection, &root_path, &rel_path)
        }
        Action::Log => log::log(&connection),
        Action::Checkout { hash } => checkout::checkout(&connection, hash),
        Action::Reset => reset::reset(&connection, &root_path),
        Action::Tree { hash } => tree::tree(&connection, hash),
    }
}
