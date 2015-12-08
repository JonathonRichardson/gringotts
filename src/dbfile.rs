use std::error::Error;
use std::io::prelude::*;
use std::io;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

pub struct Dbfile {
    file: File,
    stringPath: String
}

trait HeaderData {
    fn get_version() -> u8;
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

    pub fn read_file(mut self) {
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
}
