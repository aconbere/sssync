use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};
use rusqlite::Connection;

use crate::actions::{
    add, branch, clone, commit, diff, init, log, merge, migration, remote,
    reset, status, tree,
};
use crate::db::repo_db_path;
use crate::store::get_root_path;
use crate::types::remote_kind::RemoteKind;

#[derive(Parser, Debug)]
#[command(name = "sssync")]
#[command(author = "Anders Conbere<anders@conbere.org>")]
#[command(version = "0.1")]
#[command(about = "Keep big files in sync in S3", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug)]
pub enum Branch {
    /// Add a branch to the repository
    Add { name: String, hash: Option<String> },

    /// Switch to the branch [name]
    Switch { name: String },

    /// List all branches
    List,

    /// Show the current branch
    Show,

    /// Switch to the branch [name]
    Set { hash: String },
}

#[derive(Subcommand, Debug)]
pub enum Migration {
    /// List all migrations
    List,
    /// Show the status of migration [id]
    Show { id: String },
}

#[derive(Subcommand, Debug)]
pub enum Remote {
    /// Add a remote to the repository
    Add {
        /// Name of the remote
        name: String,

        /// Only s3 for now
        #[arg(long)]
        #[arg(value_enum)]
        kind: RemoteKind,

        /// URL referencing where the remote will exist
        #[arg(long)]
        location: String,
    },
    /// List all the remotes
    List,

    /// Initialize the remote
    Init {
        /// Name of the remote to initialize
        name: String,

        #[arg(long)]
        force: bool,
    },

    /// Push latest change to the remote
    Push { name: String },

    /// Fetches just the remote database
    FetchRemoteDB { name: String },

    /// Pushes just the remote database:
    /// Useful when needing to patch up the database with changes
    PushRemoteDB {
        name: String,

        #[arg(long)]
        force: bool,
    },

    /// Fetch the remote database and objects
    Fetch { name: String },

    /// Remove a remote
    Remove { name: String },

    /// Locate a local file in the remote
    Locate {
        /// Remote to look into
        name: String,

        /// Local file path to find the remote location of
        #[arg(long)]
        path: PathBuf,
    },

    /// List all remote branches
    Branches { name: String },
}

#[derive(Subcommand, Debug)]
pub enum Action {
    /// Subcommands to manage remotes
    Remote {
        #[command(subcommand)]
        action: Remote,
    },

    /// Subcommands to manage branches
    Branch {
        #[command(subcommand)]
        action: Branch,
    },

    /// Subcommands to manage migrations
    Migration {
        #[command(subcommand)]
        action: Migration,
    },

    /// Initialize a new repository
    Init { path: PathBuf },

    /// Add files to be staged
    Add { path: PathBuf },

    /// Commit changes to a repository
    Commit,

    /// Clone the remote located at [url] to destination [path]
    Clone {
        /// location of the remote. For now only supports s3 urls
        url: String,

        /// Destination to clone into
        path: PathBuf,
    },

    /// Show the list of commits starting at HEAD
    Log {
        #[arg(long)]
        hash: Option<String>,

        #[arg(long)]
        branch: Option<String>,

        #[arg(long)]
        remote: Option<String>,
    },

    /// Show the status of the repository
    Status,

    /// Print a representation of the tree at hash
    Tree { hash: String },

    /// Print what files are different between HEAD and [hash]
    Diff { hash: String },

    /// Clears currently staged changes
    Reset,

    Merge {
        branch: String,
        remote: Option<String>,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let pwd = Path::new(".").canonicalize()?;

    // Init isn't expected to be run with a valid root_path. We're special
    // casing init so that we can provide convenient access to root_path for
    // all the other commands.
    if let Action::Init { path } = &cli.action {
        init::init(path)?;
        return Ok(());
    }

    // Clone isn't expected to be run with a valid root_path. We're special
    // casing init so that we can provide convenient access to root_path for
    // all the other commands.
    if let Action::Clone { url, path } = &cli.action {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(clone::clone(url, path))?;
        return Ok(());
    }

    let root_path = get_root_path(&pwd)
        .ok_or(anyhow!("not in a sssync'd directory: {}", pwd.display()))?;
    let connection = Connection::open(repo_db_path(root_path))?;

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
                    root_path,
                    name,
                    *force,
                ))?;
                Ok(())
            }
            Remote::Push { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::push(&connection, root_path, name))?;
                Ok(())
            }
            Remote::Fetch { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::fetch(&connection, root_path, name))?;
                Ok(())
            }
            Remote::FetchRemoteDB { name } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::fetch_remote_database(
                    &connection,
                    root_path,
                    name,
                ))?;
                Ok(())
            }
            Remote::PushRemoteDB { name, force } => {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(remote::push_remote_database(
                    &connection,
                    root_path,
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
            Remote::Branches { name } => {
                remote::branch_list(&connection, root_path, name)?;
                Ok(())
            }
        },
        Action::Branch { action } => match action {
            Branch::Add { name, hash } => {
                branch::add(&connection, name, hash.clone())
            }
            Branch::Switch { name } => {
                branch::switch(&connection, root_path, name)
            }
            Branch::List => branch::list(&connection),
            Branch::Show => branch::show(&connection),
            Branch::Set { hash } => branch::set(&connection, &hash),
        },
        Action::Migration { action } => match action {
            Migration::List {} => {
                migration::list(&connection)?;
                Ok(())
            }
            Migration::Show { id } => migration::show(&connection, id),
        },
        Action::Commit => commit::commit(&connection, root_path),
        Action::Clone { url, path } => {
            println!("Action::Clone {} {}", url, path.display());
            Ok(())
        }
        Action::Status => {
            status::status(&connection, root_path)?;
            Ok(())
        }
        Action::Init { path } => {
            // This isn't expected to be run ever, it's special cased at the
            // start but keeping it here means we still get type
            // checking on enum coverage.
            println!("Action::Init {}", path.display());
            Ok(())
        }
        Action::Add { path } => {
            let cp = path.canonicalize()?;
            let rel_path = cp.strip_prefix(root_path)?;
            add::add(&connection, root_path, rel_path)
        }
        Action::Log {
            hash,
            branch,
            remote,
        } => log::log(
            &connection,
            root_path,
            hash.clone(),
            branch.clone(),
            remote.clone(),
        ),
        Action::Diff { hash } => diff::diff(&connection, hash),
        Action::Reset => reset::reset(&connection, root_path),
        Action::Tree { hash } => tree::tree(&connection, hash),
        Action::Merge { branch, remote } => {
            merge::merge(&connection, root_path, branch, remote)
        }
    }
}
