use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use rusqlite::Connection;
use tokio;

use crate::actions::{
    add, branch, checkout, clone, commit, init, log, migration, remote, reset,
    status, tree, update,
};
use crate::db::repo_db_path;
use crate::store::get_root_path;
use crate::types::remote_kind::RemoteKind;

#[derive(Parser, Debug)]
#[command(name = "sssync")]
#[command(author = "Anders Conbere<anders@conbere.org>")]
#[command(version = "0.1")]
#[command(about = "Keep big files in sync in S3", long_about = None)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Branch {
    Add { name: String },
    Switch { name: String },
    List,
}

#[derive(Subcommand, Debug)]
pub enum Migration {
    List,
    Show { id: String },
}

#[derive(Subcommand, Debug)]
pub enum Remote {
    Add {
        name: String,

        #[arg(long)]
        #[arg(value_enum)]
        kind: RemoteKind,

        #[clap(long)]
        location: String,
    },
    List,
    Init {
        name: String,

        #[arg(long)]
        force: bool,
    },
    Push {
        name: String,
    },
    FetchRemoteDB {
        name: String,
    },
    PushRemoteDB {
        name: String,

        #[arg(long)]
        force: bool,
    },
    Remove {
        name: String,
    },
    Locate {
        name: String,

        #[arg(long)]
        path: PathBuf,
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
    Update {
        remote: String,
    },
    Init {
        path: PathBuf,
    },
    Add {
        path: PathBuf,
    },
    Tree {
        hash: String,
    },
    Clone {
        url: String,
        path: PathBuf,
    },
    Remote {
        #[clap(subcommand)]
        action: Remote,
    },
    Branch {
        #[clap(subcommand)]
        action: Branch,
    },
    Migration {
        #[clap(subcommand)]
        action: Migration,
    },
}

pub fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    let pwd = Path::new(".").canonicalize()?;

    // Init isn't expected to be run with a valid root_path. We're special casing init so that we
    // can provide convenient access to root_path for all the other commands.
    if let Action::Init { path } = &cli.action {
        init::init(path)?;
        return Ok(());
    }

    // Clone isn't expected to be run with a valid root_path. We're special casing init so that we
    // can provide convenient access to root_path for all the other commands.
    if let Action::Clone { url, path } = &cli.action {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(clone::clone(url, path))?;
        return Ok(());
    }

    let root_path = get_root_path(&pwd)
        .ok_or(format!("not in a sssync'd directory: {}", pwd.display()))?;
    let connection = Connection::open(repo_db_path(&root_path))?;

    match &cli.action {
        Action::Remote { action } => match action {
            Remote::Add {
                name,
                kind,
                location,
            } => {
                println!("Adding remote: {}", name);
                remote::add(&connection, name, kind, location)
            }
            Remote::List => remote::list(&connection),
            Remote::Init { name, force } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::init(
                    &connection,
                    &root_path,
                    name,
                    *force,
                ))?;
                Ok(())
            }
            Remote::Push { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::push(&connection, &root_path, name))?;
                Ok(())
            }
            Remote::FetchRemoteDB { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::fetch_remote_database(
                    &connection,
                    &root_path,
                    name,
                ))?;
                Ok(())
            }
            Remote::PushRemoteDB { name, force } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::push_remote_database(
                    &connection,
                    &root_path,
                    name,
                    *force,
                ))?;
                Ok(())
            }
            Remote::Remove { name } => {
                remote::remove(&connection, name)?;
                Ok(())
            }
            Remote::Locate { name, path } => {
                remote::locate(&connection, name, path)?;
                Ok(())
            }
        },
        Action::Branch { action } => match action {
            Branch::Add { name } => branch::add(&connection, name, None),
            Branch::Switch { name } => {
                branch::switch(&connection, &root_path, name)
            }
            Branch::List => branch::list(&connection),
        },
        Action::Migration { action } => match action {
            Migration::List {} => {
                migration::list(&connection)?;
                Ok(())
            }
            Migration::Show { id } => migration::show(&connection, id),
        },
        Action::Commit => commit::commit(&connection, &root_path),
        Action::Clone { url, path } => {
            println!("Action::Clone {} {}", url, path.display());
            Ok(())
        }
        Action::Status => {
            status::status(&connection, &root_path)?;
            Ok(())
        }
        Action::Init { path } => {
            // This isn't expected to be run ever, it's special cased at the start
            // but keeping it here means we still get type checking on enum coverage.
            println!("Action::Init {}", path.display());
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
        Action::Update { remote } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(update::update(&connection, &root_path, remote))?;
            Ok(())
        }
    }
}
