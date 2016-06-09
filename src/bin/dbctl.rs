extern crate ansi_term;
extern crate env_logger;
extern crate getopts;
extern crate gringotts;

use ansi_term::Colour::*;
use getopts::Options;
use gringotts::*;
use std::env;
use std::path::Path;

fn main() {
    // Initialize the environment logger;
    env_logger::init().unwrap();

    // Grab arguments, program and command name;
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let command = args[1].clone();

    // Initialize the Options object
    let mut opts = Options::new();

    // Add a help flag
    opts.optflag("h", "help", "print this help menu");

    // Add the most important flag, to ID which database to work on.
    opts.reqopt("f", "database-file", "Specify the path to the database file to use.", "FILE");

    // Compare the matches
    let matches = match opts.parse(&args[2..]) {
        Ok(m) => { m }
        Err(f) => {
            let message = Red.bold().paint(f.to_string());
            println!("{}", message);
            println!("");
            print_usage(&program, opts);
            return; // Exit without continuing
        }
    };

    // If the user asked for help, give it to them.
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    // Grab the indicated filename.
    let filename = match matches.opt_str("f") {
        Some(val) => val,
        _ => panic!("You must indicate the database file you with to work with.")
    };

    match command.as_ref() {
        "create"    => create_db(filename),
        "info"      => get_info(filename),
        cmd => {
            let message = format!("{} is not a recognized command.", cmd);
            println!("{}", Red.bold().paint(message));
            println!("");
            print_usage(&program, opts);
        },
    }
}

fn create_db(filename: String) {
    dbfile::Dbfile::create(&filename);
    println!("Successfully created database: {}", Path::new(&filename).display());
}

fn get_info(filename: String) {
    let mut file = dbfile::Dbfile::open(&filename);
    let mut result = file.read_segment(dbfile::Locations::Version);

    println!("Filename: {}", filename);

    let version = result.get_version();
    println!("Version: {}.{}.{}", version.major, version.minor, version.build);
    println!("Block Size: {}kb", file.get_block_size());
    println!("Number of Blocks: {}", file.get_number_of_blocks());
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} COMMAND [options]", program);
    print!("{}", opts.usage(&brief));
}
