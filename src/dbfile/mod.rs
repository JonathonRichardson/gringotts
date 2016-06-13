use abomonation::{encode, decode};
use std::error::Error;
use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;
use std::mem;
use std::path::Path;
use version::*;

pub mod block;
use dbfile::block::*;

mod keychain;
use dbfile::keychain::*;

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

    pub fn get_bytes(&mut self) -> Vec<u8> {
        return self.value.clone();
    }
}

const START_OF_BLOCKS: u64 = 256;

pub enum Locations {
    MagicString,
    Version,
    BlockSize,
    NumBlocks
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
            Locations::BlockSize => ReadLocation {
                start: 106,
                length: 1,
                value_type: ReadLocationType::ByteSequence
            },
            Locations::NumBlocks => ReadLocation {
                start: 110,
                length: 8,
                value_type: ReadLocationType::ByteSequence
            }
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

const DEFAULT_BLOCK_SIZE: u8 = 4;

fn get_magic_string() -> String {
    return "GringottsDBFile - https://github.com/JonathonRichardson/gringotts".to_string();
}

impl Dbfile {
    pub fn create(string_path: &String) -> io::Result<Dbfile> {
        // Create a path to the desired file
	    let path = Path::new(&string_path);
	    let display = path.display();

        let mut file: File = match OpenOptions::new().read(true).write(true).create(true).open(string_path) {
            Ok(file) => file,
            Err(why) => {
                return Err(why);
            },
        };

        match file.write(get_magic_string().as_bytes()) {
            Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
            Ok(_) => debug!("Successfully wrote value"),
        }

        let mut dbfile = Dbfile {
            file: file,
            string_path: string_path.clone()
        };

        dbfile.write_segment(Locations::Version, CURRENT_DB_VERSION.to_bytes());
        dbfile.set_block_size(DEFAULT_BLOCK_SIZE);

        // Initialize the first block
        let mut block = dbfile.new_block();
        block.set_block_type(BlockType::Root);
        dbfile.write_block(&mut block);

        return Ok(dbfile);
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
	        Ok(_) => debug!("Successfully seeked to pos: {}", start.to_string()),
	    }

        let mut buffer = vec![0; length];

	    match self.file.read(&mut buffer) {
	        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully read value: {:?}", buffer),
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

            if (current_index <= value.len() - 1) {
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
	        Ok(_) => debug!("Successfully seeked to pos: {}", start.to_string()),
	    }

	    match self.file.write(&value_to_write) {
	        Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully wrote value: {:?}", value_to_write),
	    }
    }

    pub fn get_block_size(&mut self) -> u8 {
        let mut result = self.read_segment(Locations::BlockSize);
        let bytes: Vec<u8> = result.get_bytes();

        //let Some((result, remaining)) = unsafe { decode::<(u64, String)>(&mut bytes) };

        let array: u8 = unsafe { mem::transmute(bytes[0]) };
        return array;
    }

    pub fn get_number_of_blocks(&mut self) -> u64 {
        let mut read_result = self.read_segment(Locations::NumBlocks);
        let mut bytes: Vec<u8> = read_result.get_bytes();

        if let Some(result) = unsafe { decode::<u64>(&mut bytes) } {
            return result.0.clone();
        }
        else {
            return 0;
        }
    }

    pub fn get_block(&mut self, block_number: u64) -> Block {
        let block_size_in_bytes = (self.get_block_size() as u64) * 1024;
        let start_pos = ((block_number - 1) * block_size_in_bytes) + START_OF_BLOCKS;

        let path = Path::new(&self.string_path);
	    let display = path.display();

        match self.file.seek(SeekFrom::Start(start_pos)) {
	        Err(why) => panic!("couldn't seek on {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully seeked to pos: {}", start_pos.to_string()),
	    }

        let mut buffer = Vec::with_capacity(block_size_in_bytes as usize);
        unsafe{ buffer.set_len(block_size_in_bytes as usize) };

	    match self.file.read(&mut buffer) {
	        Err(why) => panic!("couldn't read {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully read block: {}", block_number),
	    }

        return Block::deserialize(block_number, buffer);
    }

    pub fn write_block(&mut self, block: &mut Block) {
        let block_size_in_bytes = (self.get_block_size() as u64) * 1024;
        let start_pos = ((block.blocknumber - 1) * block_size_in_bytes) + START_OF_BLOCKS;

        let path = Path::new(&self.string_path);
	    let display = path.display();

        match self.file.seek(SeekFrom::Start(start_pos)) {
	        Err(why) => panic!("couldn't seek on {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully seeked to pos: {}", start_pos.to_string()),
	    }

        match self.file.write(&mut block.serialize()) {
        	Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully wrote block: {}", block.blocknumber),
        }
    }

    fn new_block(&mut self) -> Block {
        let bytes = Vec::with_capacity(self.get_block_size() as usize);

        let old_num_blocks = self.get_number_of_blocks();
        let new_num_blocks = old_num_blocks + 1;

        // Create the new block
        let mut block = Block::deserialize(new_num_blocks, bytes);

        self.set_number_of_blocks(new_num_blocks);
        self.write_block(&mut block);

        return block;
    }

    fn set_number_of_blocks(&mut self, number: u64) {
        debug!("Setting number of blocks to: {}", number);
        let mut bytes = Vec::new();
        unsafe { encode(&number, &mut bytes); }

        //let bytes: [u8; 8] = unsafe { mem::transmute(number) };
        debug!("Setting number of blocks to: {:?}", bytes);
        self.write_segment(Locations::NumBlocks, bytes.to_vec());
    }

    fn set_block_size(&mut self, size: u8) {
        let bytes = vec!(size);
        self.write_segment(Locations::BlockSize, bytes);
    }

    pub fn set_val(&mut self, key: String, val: String) {
        let keychain = KeyChain::parse(&key);
        let key = keychain.get_final_key();
        let mut block = self.get_block_from_ref(keychain, true).unwrap();
        block.set(key, val);
        self.write_block(&mut block);
    }

    fn get_block_inner(&mut self, keys: &mut Vec<String>, blocknum: u64, create_path: bool) -> Option<Block> {
        let mut block = self.get_block(blocknum);
        let key = match keys.pop() {
            Some(s) => s,
            None => return Some(block)
        };

        return match block.get_block_ref(key.clone()) {
            Some(b) => self.get_block_inner(keys, b, create_path),
            None if create_path => {
                let new_block = self.new_block();
                block.set_block_type(BlockType::Root);
                self.write_block(&mut block);
                block.set_block_ref(key, new_block.blocknumber);
                self.write_block(&mut block);
                return self.get_block_inner(keys, new_block.blocknumber, create_path);
            },
            None => None
        };
    }

    pub fn get_block_from_ref(&mut self, keychain: KeyChain, create_path: bool) -> Option<Block> {
        let mut vec = keychain.as_vec();
        vec.reverse();
        return self.get_block_inner(&mut vec, 1, create_path);
    }

    pub fn get_val(&mut self, key: String) -> Option<String> {
        let keychain = KeyChain::parse(&key);
        let key = keychain.get_final_key();

        let mut block = self.get_block_from_ref(keychain, false);
        return match block {
            Some(b) => b.get(key),
            //Some(b) => return None,
            None => return None,
        };
    }
}
