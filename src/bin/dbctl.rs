extern crate gringotts;

use gringotts::*;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        panic!("You must supply two arguments, a command and a filename.");
    }

    let command = &args[1].clone();
    let filename = args[2].clone();

    match command.as_ref() {
        "create" => create_db(filename),
        "info"   => get_info(filename),
        _        => panic!("Unrecognized command"),
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
}
