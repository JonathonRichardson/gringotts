extern crate gringotts;

use gringotts::*;
use std::env;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        println!("The first argument is {}", args[1]);
    }

    let filename = &args[1].clone();
    let mut file = gringotts::dbfile::Dbfile::open(filename);
    file.read_file();

    let seg = &gringotts::dbfile::ReadLocation {
        start: 3,
        length: 3,
        valueType: gringotts::dbfile::ReadLocationType::string
    };

    let write_value = "XXX".to_string();

    let mut return_value = file.read_segment(seg);
    file.write_segment(seg, write_value);
    return_value = file.read_segment(seg);
}
