use std::mem;
use abomonation::{encode, decode};

pub mod kvset;

pub struct Block {
    bytes: Vec<u8>,
    pub blocknumber: u64
}

enum Section {
    Body,
    BodySize
}

impl Section {
    fn get_range(&self) -> [u64; 2] {
        match *self {
            Section::Body => [256,0],
            Section::BodySize => [0,4]
        }
    }
}

impl Block {
    pub fn new_block(blocksequencenumber: u64, block_size: u8) -> Block {
        let size: usize = ((block_size as u64) * 1024) as usize;
        return Block {
            bytes: Vec::with_capacity(size),
            blocknumber: blocksequencenumber
        }
    }

    pub fn from_bytes(blocksequencenumber: u64, bytes_vec: Vec<u8>) -> Block {
        return Block {
            bytes: bytes_vec,
            blocknumber: blocksequencenumber
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.bytes.clone();
    }

    fn read_section(&self, section: Section) -> Vec<u8> {
        let range = section.get_range();
        let mut results: Vec<u8> = Vec::new();

        let start = range[0];
        let end = match range[1] {
            // For the body, we'll indicate End of Block by "0".
            0 => (self.body_length() as u64) + start,
            _ => range[1],
        };

        for i in start..end {
            results.push(self.bytes[i as usize]);
        }

        return results;
    }

    pub fn body_length(&self) -> u32 {
        let mut bytes = self.read_section(Section::BodySize);
        if let Some(result) = unsafe { decode::<u32>(&mut bytes) } {
            return result.0.clone();
        }
        else {
            return 0;
        }
    }
}
