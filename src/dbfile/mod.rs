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

pub struct Dbfile {
    file: File,
    string_path: String
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

        let mut dbfile = Dbfile {
            file: file,
            string_path: string_path.clone()
        };

        let mut header_block = HeaderBlock::new();
        debug!("Header block serialized: {:?}", header_block.serialize());
        dbfile.write_header_block(&mut header_block);

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
        let invalid_file = || panic!("{} is not a valid Gringotts database.", display);

        // Open the file
        let mut file = OpenOptions::new().read(true).write(true).open(string_path).unwrap();

        // Check the Magic String
        let mut buffer = vec![0; MAGIC_STRING.len()];
        match file.read(&mut buffer) {
            Err(why) => panic!("Couldn't read {}: {}", display, Error::description(&why)),
            Ok(size) if size < MAGIC_STRING.len() => invalid_file(),
            Ok(_) => {
                if (String::from_utf8(buffer).unwrap() != MAGIC_STRING) {
                    invalid_file();
                }
            },
        }

        // Return a new Dbfile object
        Dbfile {
            file: file,
            string_path: string_path.clone()
        }
    }

    pub fn get_block_size(&mut self) -> u8 {
        let mut header_block = self.get_header_block();
        return header_block.get_block_size();
    }

    pub fn get_number_of_blocks(&mut self) -> u64 {
        let mut header_block = self.get_header_block();
        return header_block.get_number_of_blocks();
    }

    pub fn get_header_block(&mut self) -> HeaderBlock {
        match self.file.seek(SeekFrom::Start(0)) {
	        Err(why) => panic!("couldn't seek on: {}", Error::description(&why)),
	        Ok(_) => debug!("Successfully seeked to pos: {}", 0),
	    }

        let mut buffer = Vec::with_capacity(256 as usize);
        unsafe{ buffer.set_len(256 as usize) };

	    match self.file.read(&mut buffer) {
	        Err(why) => panic!("couldn't read: {}", Error::description(&why)),
	        Ok(_) => debug!("Successfully read block"),
	    }

        return HeaderBlock::from_bytes(buffer);
    }

    pub fn write_header_block(&mut self, block: &mut HeaderBlock) {
        let path = Path::new(&self.string_path);
	    let display = path.display();

        match self.file.seek(SeekFrom::Start(0)) {
	        Err(why) => panic!("couldn't seek on {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully seeked to pos: {}", 0),
	    }

        match self.file.write(&mut block.serialize()) {
        	Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully wrote header block"),
        }
    }

    pub fn get_block(&mut self, block_number: u64) -> NodeBlock {
        let block_size_in_bytes = (self.get_block_size() as u64) * 1024;
        let start_pos = ((block_number - 1) * block_size_in_bytes) + self::block::types::header::HEADER_BLOCK_SIZE;;

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

        return match NodeBlock::from_bytes(block_number, buffer) {
            Ok(block) => block,
            Err(_) => panic!("Couldn't get block"),
        };
    }

    pub fn write_block<T: SerializeableBlock>(&mut self, block: &mut T) {
        let block_size_in_bytes = (self.get_block_size() as u64) * 1024;
        let start_pos = ((block.get_block_number() - 1) * block_size_in_bytes) + self::block::types::header::HEADER_BLOCK_SIZE;

        let path = Path::new(&self.string_path);
	    let display = path.display();

        match self.file.seek(SeekFrom::Start(start_pos)) {
	        Err(why) => panic!("couldn't seek on {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully seeked to pos: {}", start_pos.to_string()),
	    }

        match self.file.write(&mut block.serialize()) {
        	Err(why) => panic!("couldn't write {}: {}", display, Error::description(&why)),
	        Ok(_) => debug!("Successfully wrote block: {}", block.get_block_number()),
        }
    }

    fn new_block(&mut self) -> NodeBlock {
        let bytes = Vec::with_capacity(self.get_block_size() as usize);

        let old_num_blocks = self.get_number_of_blocks();
        let new_num_blocks = old_num_blocks + 1;

        // Create the new block
        let mut block = match NodeBlock::from_bytes(new_num_blocks, bytes) {
            Ok(b) => b,
            Err(_) => panic!("Error creating new block"),
        };

        self.set_number_of_blocks(new_num_blocks);
        self.write_block(&mut block);

        return block;
    }

    fn set_number_of_blocks(&mut self, number: u64) {
        let mut header_block = self.get_header_block();
        header_block.set_number_of_blocks(number);
        self.write_header_block(&mut header_block);
    }

    fn set_block_size(&mut self, size: usize) {
        let mut header_block = self.get_header_block();
        return header_block.set_block_size(size);
    }

    pub fn set_val(&mut self, key: &String, val: String) {
        let keychain = KeyChain::parse(&key);
        let key = keychain.get_final_key();
        let mut block = self.get_block_from_ref(keychain, true).unwrap();

        match block.set(&key, val.clone()) {
            Err(e) => {
                let mut new_block = self.split_block(&mut block);

                match block.get_last_key() {
                    Some(k) => {
                        if (key > k) {
                            new_block.set(&key, val);
                            self.write_block(&mut new_block);
                        }
                        else {
                            block.set(&key, val);
                        }
                    },
                    _ => {
                        block.set(&key, val);
                        ();
                    },
                }
            },
            Ok(_) => {}
        }

        self.write_block(&mut block);
    }

    fn navigate_block_level(&mut self, block: NodeBlock, key: &String) -> NodeBlock {
        let mut next_block = block;
        'toTheRight: loop {
            let last_key = match next_block.get_last_key() {
                Some(k) => k,
                None => "".to_string(),
            };
            match next_block.get_last_key() {
                Some(k) => {
                    if (key > &k) {
                        match next_block.get_right_block() {
                            Some(n) => {
                                next_block = self.get_block(n);
                            },
                            None => break 'toTheRight
                        }
                    }
                    else {
                        break 'toTheRight;
                    }
                },
                None => break 'toTheRight
            }
        }
        return next_block;
    }

    fn get_block_inner(&mut self, keys: &mut Vec<String>, blocknum: u64, create_path: bool) -> Option<NodeBlock> {
        let mut block = self.get_block(blocknum);
        let key = match keys.pop() {
            Some(s) => s,
            None => return Some(block)
        };

        block = self.navigate_block_level(block, &key);

        debug!("Checking block: {}", block.get_block_number());
        return match block.get_block_ref(&key) {
            Some(b) => self.get_block_inner(keys, b, create_path),
            None if create_path => {
                let new_block = self.new_block();
                block.set_block_type(BlockType::Root);
                self.write_block(&mut block);
                block.set_block_ref(&key, new_block.get_block_number());
                self.write_block(&mut block);
                return self.get_block_inner(keys, new_block.get_block_number(), create_path);
            },
            None => None
        };
    }

    pub fn get_block_from_ref(&mut self, keychain: KeyChain, create_path: bool) -> Option<NodeBlock> {
        let mut vec = keychain.as_vec();
        vec.reverse();
        let mut baseblock = self.get_block_inner(&mut vec, 1, create_path);
        match baseblock {
            Some(b) => {
                return Some(self.navigate_block_level(b, &keychain.get_final_key()));
            },
            None => {
                return None;
            }
        };
    }

    pub fn get_val(&mut self, keystring: &String) -> Option<String> {
        let keychain = KeyChain::parse(keystring);
        let key = keychain.get_final_key();

        let mut block = self.get_block_from_ref(keychain, false);
        return match block {
            Some(b) => {
                return b.get(&key);
            },
            //Some(b) => return None,
            None => return None,
        };
    }

    fn split_block(&mut self, block: &mut NodeBlock) -> NodeBlock {
        let kvset = block.split();
        let mut new_block = self.new_block();

        block.set_right_block(new_block.get_block_number());
        new_block.set_kvset(kvset);

        self.write_block( block);
        self.write_block(&mut new_block);

        return new_block;
    }

    pub fn get_version(&mut self) -> Version {
        return self.get_header_block().get_version();
    }
}
