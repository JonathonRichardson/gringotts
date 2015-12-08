extern crate gringotts;

use gringotts::*;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        println!("The first argument is {}", args[1]);
    }

    let filename = &args[1].clone();
    let file = gringotts::dbfile::Dbfile::open(filename);
    file.read_file();
}
