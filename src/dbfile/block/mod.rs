struct Block {
    bytes: Vec<u8>,
    blocknumber: u64
}

enum Sections {
    Body,
    BodySize
}

impl Section {
    fn get_range() -> [u64; 2] {
        match *self {
            Section::Body => [256,0],
            Section::BodySize => [0,4]
        }
    }
}

impl Block {
    pub fn from_bytes(blocksequencenumber: u64, bytesVec: <u8>) -> block {
        return mut Block {
            bytes: bytesVec,
            blocknumber: blocksequencenumber
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        return self.bytes.clone();
    }

    fn read_section(&self, section: Section) -> Vec<u8> {
        let range = section.get_range();
        let mut results = Vec::new();

        for(let i = range[0]; i < range[1]; i++) {
            results.push(self.bytes[i]);
        }

        return results;
    }

    pub fn body_length(&self) -> u32 {
        let bytes = self.read_section(Sections::BodySize);
        let size: u32 = unsafe { mem::transmute(bytes) };
    }
}
