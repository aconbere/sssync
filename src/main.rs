use std::process::exit;

use crate::cli::run;

mod actions;
mod cli;
mod db;
mod hash;
mod models;
mod remote;
mod store;
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
