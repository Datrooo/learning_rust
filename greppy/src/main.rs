use std::{env, process};

use greppy;
use greppy::Config;

/*
    TODO: 
        1)поменять clone на что-то более эффективное
        2) добавить разбор регулярных выражений???
        3) мб иначе завершпть програму?)
*/

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Failed to parse config {}", err);
        process::exit(1);
    });
    println!("{:?}", args);
    println!("Searching for {}", config.query);
    println!("in file {}", config.filename);

    if let Err(e) = greppy::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }}

