use std::mem;
use abomonation::{encode, decode};

pub mod kvset;

pub struct Block {
    bytes: Vec<u8>,
    pub blocknumber: u64
}

pub trait HasSectionAddress {
    fn get_start_and_end(&self) -> [u64; 2];
}

enum CommonSection {
    Body,
    BodySize
}

impl HasSectionAddress for CommonSection {
    fn get_start_and_end(&self) -> [u64; 2] {
        match *self {
            CommonSection::Body => [256,0],
            CommonSection::BodySize => [0,4]
        }
    }
}

pub trait SerializeableBlock {
    fn deserialize(block_number: u64, bytes_vec: Vec<u8>)  -> Block;
    fn serialize(&self) -> Vec<u8>;
}

impl SerializeableBlock for Block {
    fn deserialize(block_number: u64, bytes_vec: Vec<u8>) -> Block {
        return Block {
            bytes: bytes_vec.clone(),
            blocknumber: block_number
        }
    }

    fn serialize(&self) -> Vec<u8> {
        return self.bytes.clone();
    }
}

trait DBBlock {
    fn read_section<T>(&self, section: T) -> Vec<u8> where T: HasSectionAddress;
    fn write_section<T>(&mut self, section: T, bytes: Vec<u8>) where T: HasSectionAddress;
    fn body_length(&self) -> u32;
}

impl DBBlock for Block {
    fn read_section<T>(&self, section: T) -> Vec<u8> where T: HasSectionAddress{
        let range = section.get_start_and_end();
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

    fn write_section<T>(&mut self, section: T, bytes: Vec<u8>) where T: HasSectionAddress {
        let range = section.get_start_and_end();
        let start = range[0];
        let end = match range[1] {
            // For the body, we'll indicate End of Block by "0".
            0 => (self.body_length() as u64) + start,
            _ => range[1],
        };

        for i in start..end {
            if (i > ((bytes.len() as u64) + 1)) {
                self.bytes[i as usize] = 0;
            }
            else {
                self.bytes[i as usize] = bytes[i as usize];
            }
        }
    }

    fn body_length(&self) -> u32 {
        let mut bytes = self.read_section(CommonSection::BodySize);
        if let Some(result) = unsafe { decode::<u32>(&mut bytes) } {
            return result.0.clone();
        }
        else {
            return 0;
        }
    }
}

pub struct HeaderBlock {
    block: Block
}
