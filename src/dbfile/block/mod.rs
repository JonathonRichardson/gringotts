use abomonation::{encode, decode};

pub mod kvset;
use dbfile::block::kvset::*;

pub struct Block {
    header_bytes: Vec<u8>,
    pub blocknumber: u64,
    pub data: KVSet,
}

pub trait HasSectionAddress {
    fn get_start_and_end(&self) -> [u64; 2];
}

enum CommonSection {
    Body,
    BodySize,
    Type
}

impl HasSectionAddress for CommonSection {
    fn get_start_and_end(&self) -> [u64; 2] {
        match *self {
            CommonSection::Body => [256,0],
            CommonSection::BodySize => [0,4],
            CommonSection::Type => [4,8],
        }
    }
}

pub enum BlockType {
    Node,
    Root,
}

impl BlockType {
    fn get_code(&self) -> u32 {
        let code: u32 = match *self {
            BlockType::Root => 22,
            BlockType::Node => 40
        };

        return code;
    }

    fn get_block_type(code: u32) -> BlockType {
        match code {
            22 => BlockType::Root,
            _ => BlockType::Node
        }
    }
}

pub trait SerializeableBlock {
    fn deserialize(block_number: u64, bytes_vec: Vec<u8>)  -> Block;
    fn serialize(&mut self) -> Vec<u8>;
}

const HEADER_SIZE: usize = 256;

impl SerializeableBlock for Block {
    fn deserialize(block_number: u64, bytes_vec: Vec<u8>) -> Block {
        let mut header_bytes = Vec::with_capacity(HEADER_SIZE);
        for i in 0..HEADER_SIZE {
            if ((i + 1) > bytes_vec.len()) { // use i+1 instead of len() - 1 to prevent issues when len() is 0
                header_bytes.push(0);
            }
            else {
                header_bytes.push(bytes_vec[i]);
            }
        }

        let mut block = Block {
            header_bytes: header_bytes,
            blocknumber: block_number,
            data: KVSet::new()
        };

        let body_length = block.body_length();
        let mut body = Vec::with_capacity(body_length as usize);
        for i in 0..body_length {
            if ((i + (HEADER_SIZE as u32) + 1) > (bytes_vec.len() as u32)) { // use i+1 instead of len() - 1 to prevent issues when len() is 0
                body.push(0);
            }
            else {
                body.push(bytes_vec[(i as usize) + HEADER_SIZE]);
            }
        }
        block.data = match KVSet::deserialize(&mut body) {
            Ok(set) => set,
            Err(_) => panic!("Block {} is corrupt.", block_number)
        };

        return block;
    }

    fn serialize(&mut self) -> Vec<u8> {
        let mut data_bytes = self.data.serialize();
        self.set_body_length(data_bytes.len() as u32);
        let mut serialized_bytes = self.header_bytes.clone();
        serialized_bytes.append(&mut data_bytes);
        return serialized_bytes;
    }
}

trait DBBlock {
    fn read_section<T>(&self, section: T) -> Vec<u8> where T: HasSectionAddress;
    fn write_section<T>(&mut self, section: T, bytes: Vec<u8>) where T: HasSectionAddress;
    fn body_length(&self) -> u32;
    fn set_body_length(&mut self, length: u32);
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
            results.push(self.header_bytes[i as usize]);
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
        let length = end - start;

        debug!("About to following bytes to block {} section at {}: {:?}", self.blocknumber, start, bytes);

        // Determine the last populated index of the byte vector.
        let last_index_block_data = self.header_bytes.len() as u64;
        let last_index_of_data_to_write = (bytes.len() - 1) as u64;
        debug!("Last index of block data: {}", last_index_block_data);

        // If we need to, expand the vector containing the block contents.
        // --> TODO: Eventually, this should be able to be replaced by the resize() method, but it's
        //           unstable as of now.
        if ((last_index_block_data) < end) {
            for i in (last_index_block_data)..(end + 1) {
                self.header_bytes.push(0);
            }
        }

        // Iterate over the bytes in the section
        for i in 0..length {
            // Determine the index in the block to write.
            let index_to_set = (i + start) as usize;

            // If we're beyond the bytes to write, just use zeros
            if (i > last_index_of_data_to_write) {
                self.header_bytes[index_to_set] = 0;
            }
            else {
                self.header_bytes[index_to_set] = bytes[i as usize];
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

    fn set_body_length(&mut self, length: u32) {
        let mut vector = Vec::new();
        unsafe { encode(&length, &mut vector); }
        self.write_section(CommonSection::BodySize, vector);
    }
}

pub trait BasicBlock : SerializeableBlock {
    fn set_block_type(&mut self, block_type: BlockType);
}

impl BasicBlock for Block {
    fn set_block_type(&mut self, block_type: BlockType) {
        let code: u32 = block_type.get_code();

        let mut bytes = Vec::new();
        unsafe { encode(&code, &mut bytes); }

        self.write_section(CommonSection::Type, bytes);
    }
}

impl Block {
    pub fn set(&mut self, key: String, val: String) {
        self.data.put(key, val);
    }

    pub fn get(&mut self, key: String) -> Option<&String> {
        return self.data.get(key);
    }
}
