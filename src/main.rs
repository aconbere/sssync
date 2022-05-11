use std::process::exit;

use crate::cli::run;

mod actions;
mod cli;
mod db;
mod file_entry;
mod hash;
mod store;

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}
