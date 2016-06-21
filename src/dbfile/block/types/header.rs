use abomonation::{encode, decode};

use dbfile::block::*;
use dbfile::block::sections::header::*;
use version::*;

pub const MAGIC_STRING: &'static str = "GringottsDBFile - https://github.com/JonathonRichardson/gringotts";
pub const HEADER_BLOCK_SIZE: u64 = 256;
const DEFAULT_BLOCK_SIZE: usize = 4;
const CURRENT_DB_VERSION: Version = Version {
    major: 0,
    minor: 0,
    build: 1,
};

enum HeaderSection {
    MagicString,
    Version,
    BlockSize,
    NumBlocks
}

impl HasSectionAddress for HeaderSection {
    fn get_start_and_end(&self) -> [u64; 2] {
        match *self {
            HeaderSection::MagicString => [0,  65],
            HeaderSection::Version     => [65, 71],
            HeaderSection::BlockSize   => [71, 72],
            HeaderSection::NumBlocks   => [72, 80],
        }
    }
}

pub struct HeaderBlock {
    header: BlockHeader,
    size: usize,
}

impl HeaderBlock {
    pub fn new() -> HeaderBlock {
        let mut header = BlockHeader::new();

        let mut magic_bytes = Vec::with_capacity(MAGIC_STRING.len());
        let mut magic_slice = MAGIC_STRING.as_bytes();
        for i in 0..magic_slice.len() {
            magic_bytes.push(magic_slice[i]);
        }

        header.write_section(HeaderSection::BlockSize,   vec!(DEFAULT_BLOCK_SIZE as u8));
        header.write_section(HeaderSection::MagicString, magic_bytes);
        header.write_section(HeaderSection::Version,     CURRENT_DB_VERSION.to_bytes());

        debug!("Header bytes: {:?}", header.serialize());

        return HeaderBlock {
            header: header,
            size: DEFAULT_BLOCK_SIZE
        };
    }

    pub fn from_bytes(bytes_vec: Vec<u8>) -> HeaderBlock {
        let header = BlockHeader::from_bytes(1, &bytes_vec);

        return HeaderBlock {
            header: header,
            size: DEFAULT_BLOCK_SIZE
        };
    }

    pub fn get_number_of_blocks(&self) -> u64 {
        let mut bytes: Vec<u8>  = self.header.read_section(HeaderSection::NumBlocks);

        debug!("Bytes read: {:?}", bytes);
        if let Some(result) = unsafe { decode::<u64>(&mut bytes) } {
            return result.0.clone();
        }
        else {
            return 0;
        }
    }

    pub fn set_number_of_blocks(&mut self, number: u64) {
        debug!("Setting number of blocks to: {}", number);
        let mut bytes = Vec::new();
        unsafe { encode(&number, &mut bytes); }

        //let bytes: [u8; 8] = unsafe { mem::transmute(number) };
        debug!("Setting number of blocks to: {:?}", bytes);
        self.header.write_section(HeaderSection::NumBlocks, bytes.to_vec());
    }

    pub fn get_block_size(&mut self) -> u8 {
        let mut bytes: Vec<u8>  = self.header.read_section(HeaderSection::BlockSize);

        if (bytes.len() >= 1) {
            return bytes[0];
        }
        else {
            return 0;
        }
    }

    pub fn set_block_size(&mut self, size: usize) {
        let bytes = vec!(size as u8);
        self.header.write_section(HeaderSection::BlockSize, bytes);
    }

    pub fn get_version(&self) -> Version {
        let bytes = self.header.read_section(HeaderSection::Version);
        return Version::from_bytes(bytes);
    }
}

impl SerializeableBlock for HeaderBlock {
    fn serialize(&mut self) -> Vec<u8> {
        return self.header.serialize();
    }

    fn get_block_number(&self) -> u64 {
        return 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dbfile::block::{HasSectionAddress};

    #[test]
    fn magic_string_length() {
        assert_eq!(MAGIC_STRING.len(), super::HeaderSection::MagicString.get_length());
    }
}
