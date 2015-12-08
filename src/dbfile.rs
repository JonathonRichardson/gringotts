use std::error::Error;
use std::io::prelude::*;
use std::io;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

pub enum ReadLocationType {
    number,
    string
}

pub struct ReadLocation {
    pub start: u8,
    pub length: u8,
    pub valueType: ReadLocationType
}

pub struct Dbfile {
    file: File,
    stringPath: String
}

trait HeaderData {
    fn get_version() -> u16;
}

impl Dbfile {
    pub fn open(stringPath: &String) -> Dbfile {
	    // Create a path to the desired file
	    let path = Path::new(&stringPath);
	    let display = path.display();
	
	    // Open the path in read-only mode, returns `io::Result<File>`
	    let mut file = match File::open(&path) {
	        // The `description` method of `io::Error` returns a string that
	        // describes the error
	        Err(why) => panic!("couldn't open {}: {}", display,
	                                                   Error::description(&why)),
	        Ok(file) => file,
	    };

        
        Dbfile {
            file: file,
            stringPath: stringPath.clone()
        }
    }

    pub fn read_file(&mut self) {
	    // Read the file contents into a string, returns `io::Result<usize>`
	    let mut s = String::new();
	    let path = Path::new(&self.stringPath);
	    let display = path.display();
	    match self.file.read_to_string(&mut s) {
	        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
	        Ok(_) => print!("{} contains:\n{}", display, s),
	    }
	
	    // `file` goes out of scope, and the "hello.txt" file gets closed
    }

    pub fn read_segment(&mut self, loc: ReadLocation) -> String {
        let start: u64 = loc.start as u64;
        let length: usize = loc.length as usize;
        let mut read_value: String = String::new();

	    let path = Path::new(&self.stringPath);
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

        read_value = String::from_utf8(buffer).unwrap();

        println!("Value is: {}", read_value);
        return read_value;
    }
}


impl HeaderData for Dbfile {
    fn get_version() -> u16 {
        return 0;
    }
}
