use std::{env, process};

use greppy;
use greppy::Config;

/*
    TODO: 
        1) добавить разбор регулярных выражений???
*/

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Failed to parse config {}", err);
        process::exit(1);
    });

    println!("Searching for {}", config.query);
    println!("in file {}", config.filename);

    if let Err(e) = greppy::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }}

