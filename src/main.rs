use std::process::exit;

use crate::cli::run;

mod actions;
mod cli;
mod db;
mod hash;
mod migration;
mod models;
mod remote;
mod s3;
mod store;
mod tree;
mod types;

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
