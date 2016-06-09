use std::collections::BTreeMap;
use std::str;
use error::*;

pub struct KVPair {
    key: String,
    value: String,
}

pub struct KVSet {
    data: BTreeMap<String, String>
    pointers: BTreeMap<String, u64>
}

enum Character {
    Regular(u8),
    RecordSeperator,
    ValueStart,
    PointerStart
}

impl Character {
    gn get_value(&self) -> Vec<u8> {
        match self {
            Character::Regular(0)      => vec!(0,2),
            Character::Regular(val)    => val,
            Character::RecordSeperator => vec!(0,0),
            Character::ValueStart      => vec!(0,1),
            Character::PointerStart    => vec!(0,3),
        }
    }
}

impl KVSet {
    pub fn new() -> KVSet {
        return KVSet {
            data: BTreeMap::new()
            pointers: BTreeMap::new()
        }
    }

    /// Adds a new key/value pair to the KVSet.  The return value will be a String, if there was
    /// a previous value and this is therefore an update, or None, if this was a true insert.
    pub fn put(&mut self, key: String, value: String) -> Option<String> {
        return self.data.insert(key, value); // this returns the old value or None
    }

    pub fn get(&self, key: String) -> Option<&String> {
        return self.data.get(&key);
    }

    pub fn deserialize(bytes: &mut Vec<u8>) -> Result<KVSet, CorruptDataError> {
        let mut datamap = BTreeMap::new();
        let mut pointermap = BTreeMap::new();
        let length = bytes.len();

        // If the byte vector is empty, we're good.
        if (length == 0) {
            return Ok(KVSet {
                data: map,
                pointers: BTreeMap::new()
            });
        }

        let get_character = |&mut bytes| -> Option<Character> {
            let byte = bytes.pop();

            if (byte == 0) {
                match bytes.pop() {
                    0 => return Character::RecordSeperator,
                    1 => return Character::ValueStart,
                    2 => return Character::Regular(0),
                    3 => return Character::PointerStart,
                }
            }
            else {
                return Character::Regular(byte);
            }
        };

        let get_value_vec = |&mut bytes| -> Vec<u8> {
            let value_bytes = Vec::new();
            loop {
                match get_character(&mut bytes) {
                    Some(Character::Regular(byte)) => string_bytes.push(byte),
                    Some(_) => {
                        // Put back what you have taken
                        bytes.append(_.get_value());
                        break; // we're done here.
                    },
                    None => break,
                }
            }
            return value_bytes;
        }

        bytes.reverse();
        while(bytes.len() > 0) {
            let mut key_buffer = Vec::new();
            let mut val_buffer = Vec::new();
            let mut ptr_buffer = Vec::new();

            match get_character(&mut bytes) {
                Character::ValueStart => val_buffer.append(get_value_vec(&mut bytes)),
                Character::PointerStart => ptr_buffer.append(get_value_vec(&mut bytes)),
                Character::RecordSeperator => {
                    let key = String::from_utf8(key_buffer.clone()).unwrap(); // TODO: change unwrap to throw CorruptDataError
                    let val = String::from_utf8(val_buffer.clone()).unwrap(); // TODO: change unwrap to throw CorruptDataError

                    datamap.insert(key, val);

                    match ptr_buffer.len() {
                        0 => {}, // do nothing
                        8 => {
                            let block_number = unsafe { decode::<u64>(&mut ptr_buffer) };
                            pointermap.insert(key, block_number);
                        }
                    }
                },
            }
        }

        return Ok(KVSet {
            data: map
        });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for key in self.data.keys() {
            if (bytes.len() > 0) {
                bytes.push(0);
                bytes.push(0);
            }

            for byte in key.clone().into_bytes() {
                if (byte == 0) {
                    bytes.push(0);
                    bytes.push(2);
                }
                else {
                    bytes.push(byte);
                }
            }

            bytes.push(0);
            bytes.push(1);

            for byte in self.data.get(key).unwrap().clone().into_bytes() {
                if (byte == 0) {
                    bytes.push(0);
                    bytes.push(2);
                }
                else {
                    bytes.push(byte);
                }
            }

            if (self.data.contains_key(key)) {

            }

            bytes.push(0);
            bytes.push(3);
        }

        return bytes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;


    #[test]
    fn basic_serialization() {
        let mut keyset = KVSet::new();
        keyset.put(String::from("yes"),   String::from("no"));
        keyset.put(String::from("hello"), String::from("goodbye"));

        let serialized_vector = vec!(
            104, 101, 108, 108, 111,
            0, 1,
            103, 111, 111, 100, 98, 121, 101,
            0, 0,
            121, 101, 115,
            0, 1,
            110, 111,
        );

        assert_eq!(keyset.serialize(), serialized_vector);
    }
}
