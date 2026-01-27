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
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });
    println!("{:?}", args);
    println!("Searching for {}", config.query);
    println!("in file {}", config.filename);

    if let Err(e) = greppy::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }}

