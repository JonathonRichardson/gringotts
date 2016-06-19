use error::*;
use dbfile::block::*;
use dbfile::block::kvset::*;
use dbfile::block::sections::header::*;

pub struct NodeBlock {
    header: BlockHeader,
    data: KVSet,
    size: usize,
}

pub trait DataBlock {
    fn set(&mut self, key: &String, val: String) -> Result<Option<String>, NoRoomError>;
    fn get(&self, key: &String) -> Option<String>;
    fn get_block_ref(&self, key: &String) -> Option<u64>;
    fn set_block_ref(&mut self, key: &String, blockref: u64) -> Result<Option<u64>, NoRoomError>;
    fn get_last_key(&self) -> Option<String>;
    fn set_kvset(&mut self, kvset: KVSet);
    fn split(&mut self) -> KVSet;
}

impl NodeBlock {
    pub fn from_bytes(blocknumber: u64, bytes_vec: Vec<u8>) -> Result<NodeBlock, CorruptDataError> {
        let header = BlockHeader::from_bytes(blocknumber, &bytes_vec);
        let body_length = header.body_length();
        let mut body = Vec::with_capacity(body_length as usize);
        for i in 0..body_length {
            if ((i + (HEADER_SIZE as u32) + 1) > (bytes_vec.len() as u32)) { // use i+1 instead of len() - 1 to prevent issues when len() is 0
                body.push(0);
            }
            else {
                body.push(bytes_vec[(i as usize) + HEADER_SIZE]);
            }
        }

        let msg: &str = &*(format!("Block {} is corrupt", blocknumber));

        let data = match KVSet::deserialize(&mut body) {
            Ok(set) => set,
            Err(_) => return Err(CorruptDataError::new(msg)),
        };

        return Ok(NodeBlock {
            header: header,
            data: data,
            size: bytes_vec.len()
        });
    }
}

impl DataBlock for NodeBlock {
    fn set(&mut self, key: &String, val: String) -> Result<Option<String>, NoRoomError> {
        let retval = self.data.put(key, val);

        return match(self.serialize().len() <= self.size) {
            true => Ok(retval),
            false => {
                self.data.delete(key);
                return Err(NoRoomError::new("No Room in block"));
            }
        }
    }

    fn get(&self, key: &String) -> Option<String> {
        return match self.data.get(key) {
            Some(s) => Some(s.clone()),
            None => None,
        };
    }

    fn get_block_ref(&self, key: &String) -> Option<u64> {
        return match self.data.get_block_ref(key) {
            Some(n) => Some(n.clone()),
            None => None,
        }
    }

    fn set_block_ref(&mut self, key: &String, blockref: u64) -> Result<Option<u64>, NoRoomError> {
        let retval = self.data.put_block_ref(&key, blockref);

        return match(self.serialize().len() <= self.size) {
            true => Ok(retval),
            false => {
                self.data.delete_block_ref(&key);
                return Err(NoRoomError::new("No Room in block"));
            }
        }
    }

    fn get_last_key(&self) -> Option<String> {
        return self.data.get_last_key();
    }

    fn set_kvset(&mut self, kvset: KVSet) {
        self.data = kvset;
    }

    fn split(&mut self) -> KVSet {
        return self.data.split();
    }
}

impl SerializeableBlock for NodeBlock {
    fn serialize(&mut self) -> Vec<u8> {
        let mut data_bytes = self.data.serialize();
        self.set_body_length(data_bytes.len() as u32);
        let mut serialized_bytes = self.header.serialize();
        serialized_bytes[0] = 66;
        serialized_bytes[1] = 76;
        serialized_bytes.append(&mut data_bytes);
        return serialized_bytes;
    }

    fn get_block_number(&self) -> u64 {
        return self.header.get_block_number();
    }
}

impl HasBlockHeader for NodeBlock {
    fn get_header(&mut self) -> &mut BlockHeader {
        return &mut self.header;
    }
}

impl Navigable for NodeBlock {}
impl BasicBlock for NodeBlock {}
