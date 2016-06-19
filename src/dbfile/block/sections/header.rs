use dbfile::block::*;
use abomonation::{encode, decode};

pub const HEADER_SIZE: usize = 256;

pub trait HasBlockHeader {
    fn get_header(&mut self) -> &mut BlockHeader;
}

pub struct BlockHeader {
    header_bytes: Vec<u8>,
    blocknumber: u64,
    size: usize,
}

impl BlockHeader {
    pub fn from_bytes(block_number: u64, bytes_vec: &Vec<u8>) -> BlockHeader {
        let mut header_bytes = Vec::with_capacity(HEADER_SIZE);
        for i in 0..HEADER_SIZE {
            if ((i + 1) > bytes_vec.len()) { // use i+1 instead of len() - 1 to prevent issues when len() is 0
                header_bytes.push(0);
            }
            else {
                header_bytes.push(bytes_vec[i]);
            }
        }

        let mut block = BlockHeader {
            header_bytes: header_bytes,
            blocknumber: block_number,
            size: bytes_vec.len(),
        };

        return block;
    }

    pub fn read_section<T>(&self, section: T) -> Vec<u8> where T: HasSectionAddress{
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

    pub fn write_section<T>(&mut self, section: T, bytes: Vec<u8>) where T: HasSectionAddress {
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

    pub fn body_length(&self) -> u32 {
        let mut bytes = self.read_section(CommonSection::BodySize);
        if let Some(result) = unsafe { decode::<u32>(&mut bytes) } {
            return result.0.clone();
        }
        else {
            return 0;
        }
    }

    pub fn set_body_length(&mut self, length: u32) {
        let mut vector = Vec::new();
        unsafe { encode(&length, &mut vector); }
        self.write_section(CommonSection::BodySize, vector);
    }
}

impl SerializeableBlock for BlockHeader {
    fn serialize(&mut self) -> Vec<u8> {
        return self.header_bytes.clone();
    }

    fn get_block_number(&self) -> u64 {
        return self.blocknumber;
    }
}
