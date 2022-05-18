use std::error::Error;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use tokio;

use crate::actions::{add, checkout, commit, init, log, push, remote, reset, status, tree};
use crate::db::get_connection;
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
    Push {
        remote: String,
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

    let connection = get_connection(root_path)?;

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
                println!("Remote::List: {}", path.display());
                remote::list(&connection)
            }
        },
        Action::Commit => {
            println!("Action::Commit: {}", path.display());
            commit::commit(&connection)
        }
        Action::Status => {
            println!("Action::Status: {}", path.display());
            status::status(&connection, &path)
        }
        Action::Init => {
            // handled above since it doesn't have a root_path yet
            Ok(())
        }
        Action::Add => {
            println!("Action::Add: {}", path.display());
            let rel_path = path.strip_prefix(root_path)?;
            add::add(&connection, &path, &root_path, &rel_path)
        }
        Action::Log => {
            println!("Action::Log: {}", path.display());
            log::log(&connection)
        }
        Action::Checkout { hash } => checkout::checkout(&connection, hash),
        Action::Reset => {
            println!("Action::Reset: {}", path.display());
            reset::reset(&connection, &path)
        }
        Action::Tree { hash } => {
            println!("Action::Tree: {}", path.display());
            tree::tree(&connection, hash)
        }
        Action::Push { remote } => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(push::push(&connection, root_path, remote))?;
            Ok(())
        }
    }
}
