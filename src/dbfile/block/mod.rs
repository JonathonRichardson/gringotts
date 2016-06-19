use abomonation::{encode, decode};
use error::*;

pub mod kvset;
mod sections;
pub mod types;

use self::kvset::*;
use self::sections::header::{HasBlockHeader};
pub use self::types::*;

pub trait HasSectionAddress {
    fn get_start_and_end(&self) -> [u64; 2];

    fn get_length(&self) -> usize {
        let start_and_end: [u64; 2] = self.get_start_and_end();
        return (start_and_end[1] - start_and_end[0]) as usize;
    }

    fn get_start(&self) -> u64 {
        return self.get_start_and_end()[0];
    }

    fn get_end(&self) -> u64 {
        return self.get_start_and_end()[1];
    }
}

pub enum CommonSection {
    Body,
    BodySize,
    Type,
    NextBlock
}

impl HasSectionAddress for CommonSection {
    fn get_start_and_end(&self) -> [u64; 2] {
        match *self {
            CommonSection::Body      => [256,  0],
            CommonSection::BodySize  => [2,    6],
            CommonSection::Type      => [6,   10],
            CommonSection::NextBlock => [10,  18]
        }
    }
}

pub enum BlockType {
    Header,
    Node,
    Root,
}

impl BlockType {
    fn get_code(&self) -> u32 {
        let code: u32 = match *self {
            BlockType::Root => 22,
            BlockType::Node => 40,
            BlockType::Header => 9,
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
    fn serialize(&mut self) -> Vec<u8>;
    fn get_block_number(&self) -> u64;
}

pub trait BasicBlock : HasBlockHeader {
    fn set_block_type(&mut self, block_type: BlockType) {
        let code: u32 = block_type.get_code();

        let mut bytes = Vec::new();
        unsafe { encode(&code, &mut bytes); }

        self.get_header().write_section(CommonSection::Type, bytes);
    }

    fn body_length(&mut self) -> u32 {
        let mut bytes = self.get_header().read_section(CommonSection::BodySize);
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
        self.get_header().write_section(CommonSection::BodySize, vector);
    }
}

pub trait Navigable : HasBlockHeader {
    fn set_right_block(&mut self, num: u64) {
        let mut bytes = Vec::new();
        unsafe { encode(&num, &mut bytes); }
        self.get_header().write_section(CommonSection::NextBlock, bytes);
    }

    fn get_right_block(&mut self) -> Option<u64> {
        let mut bytes = self.get_header().read_section(CommonSection::NextBlock);
        if let Some(result) = unsafe { decode::<u64>(&mut bytes) } {
            return match *result.0 {
                0 => None,
                n => Some(n)
            }
        }
        else {
            return None;
        }
    }
}
