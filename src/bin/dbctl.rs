extern crate gringotts;

use gringotts::*;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        println!("The first argument is {}", args[1]);
    }

    let filename = &args[1].clone();
    let mut file = dbfile::Dbfile::open(filename);
    file.read_file();

    let seg = &dbfile::ReadLocation {
        start: 3,
        length: 3,
        valueType: dbfile::ReadLocationType::UTF8String
    };

    let write_value = "XXX".to_string();

    file.read_segment(seg);
    file.write_segment(seg, write_value);
    file.read_segment(seg);
}
