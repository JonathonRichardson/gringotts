#![allow(unused_parens)]

use std::error::Error;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;
use version::*;

pub enum ReadLocationType {
    Number,
    UTF8String,
    ByteSequence
}

pub struct ReadLocation {
    pub start: u8,
    pub length: usize,
    pub value_type: ReadLocationType
}

pub struct ReadResult {
    value_type: ReadLocationType,
    value: Vec<u8>
}

impl ReadResult {
    pub fn get_string(&mut self) -> String {
        match self.value_type {
            ReadLocationType::UTF8String => String::from_utf8(self.value.clone()).unwrap(),
            _ => panic!("Tried to get a string off of a non-string value"),
        }
    }
    pub fn get_version(&mut self) -> Version {
        match self.value_type {
            ReadLocationType::ByteSequence => Version::from_bytes(self.value.clone()),
            _ => panic!("Tried to get a string off of a non-byte sequence value"),
        }
    }
}

pub enum Locations {
    MagicString,
    Version
}

impl Locations {
    fn get_read_location(&self) -> ReadLocation {
        match *self {
            Locations::MagicString => ReadLocation {
                start: 0,
                length: get_magic_string().len(),
                value_type: ReadLocationType::UTF8String
            },
            Locations::Version => ReadLocation {
                start: 100,
                length: 6,
                value_type: ReadLocationType::ByteSequence
            },
        }
    }
}

pub struct Dbfile {
    file: File,
    string_path: String
}

const CURRENT_DB_VERSION: Version = Version {
    major: 0,
    minor: 0,
    build: 1,
};

trait HeaderData {
    fn get_version() -> u16;
}

fn get_magic_string() -> String {
    return "GringottsDBFile - https://github.com/JonathonRichardson/gringotts".to_string();
}

impl Dbfile {
    pub fn create(string_path: &String) -> Dbfile {
        // Create a path to the desired file
	    let path = Path::new(&string_path);
	    let display = path.display();

        let mut file = OpenOptions::new().read(true).write(true).create(true).open(string_path).unwrap();

        match file.write(get_magic_string().as_bytes()) {
            Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
            Ok(_) => println!("Successfully wrote value"),
        }

        let mut dbfile = Dbfile {
            file: file,
            string_path: string_path.clone()
        };

        dbfile.write_segment(Locations::Version, CURRENT_DB_VERSION.to_bytes());

        return dbfile;
    }

    pub fn open(string_path: &String) -> Dbfile {
	    // Create a path to the desired file
	    let path = Path::new(&string_path);
	    let display = path.display();

        let mut file = OpenOptions::new().read(true).write(true).open(string_path).unwrap();

        let magic_string = get_magic_string();
        let mut buffer = vec![0; magic_string.len()];

        let invalid_file = || panic!("{} is not a valid Gringotts database.", display);

        match file.read(&mut buffer) {
            Err(why) => panic!("Couldn't read {}: {}", display, Error::description(&why)),
            Ok(size) if size < magic_string.len() => invalid_file(),
            Ok(_) => {},
        }

        if (String::from_utf8(buffer).unwrap() != magic_string) {
            invalid_file();
        }

        Dbfile {
            file: file,
            string_path: string_path.clone()
        }
    }

    pub fn read_file(&mut self) {
	    // Read the file contents into a string, returns `io::Result<usize>`
	    let mut s = String::new();
	    let path = Path::new(&self.string_path);
	    let display = path.display();
	    match self.file.read_to_string(&mut s) {
	        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
	        Ok(_) => print!("{} contains:\n{}", display, s),
	    }

	    // `file` goes out of scope, and the "hello.txt" file gets closed
    }

    pub fn read_segment(&mut self, loc: Locations) -> ReadResult {
        let loc = loc.get_read_location();
        let start: u64 = loc.start as u64;
        let length: usize = loc.length as usize;

	    let path = Path::new(&self.string_path);
	    let display = path.display();

	    match self.file.seek(SeekFrom::Start(start)) {
	        Err(why) => panic!("couldn't seek on {}: {}", display, Error::description(&why)),
	        Ok(_) => println!("Successfully seeked to pos: {}", start.to_string()),
	    }

        let mut buffer = vec![0; length];

	    match self.file.read(&mut buffer) {
	        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
	        Ok(_) => println!("Successfully read value"),
	    }

        ReadResult {
            value: buffer,
            value_type: loc.value_type
        }
    }

    pub fn write_segment(&mut self, loc: Locations, value: Vec<u8>) {
        let loc = loc.get_read_location();
        let start: u64 = loc.start as u64;
        let length: usize = loc.length as usize;


        let mut value_to_write: Vec<u8> = Vec::new();
        while (value_to_write.len() < length) {
            let current_index = value_to_write.len();

            if (current_index >= value.len() + 1) {
                value_to_write.push(value[current_index]);
            }
            else {
                value_to_write.push(0);
            }
        }

	    let path = Path::new(&self.string_path);
	    let display = path.display();

	    match self.file.seek(SeekFrom::Start(start)) {
	        Err(why) => panic!("couldn't seek on {}: {}", display, Error::description(&why)),
	        Ok(_) => println!("Successfully seeked to pos: {}", start.to_string()),
	    }

	    match self.file.write(&value_to_write) {
	        Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
	        Ok(_) => println!("Successfully wrote value"),
	    }
    }
}


impl HeaderData for Dbfile {
    fn get_version() -> u16 {
        return 0;
    }
}
