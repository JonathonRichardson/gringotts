use abomonation::{encode, decode};
use std::collections::BTreeMap;
use error::*;

pub struct KVSet {
    data: BTreeMap<String, String>,
    pointers: BTreeMap<String, u64>
}

enum Character {
    Regular(u8),
    RecordSeperator,
    ValueStart,
    PointerStart
}

fn get_character(bytes: &mut Vec<u8>) -> Option<Character> {
    if (bytes.len() == 0) {
        return None;
    }

    let byte = bytes.pop();

    match byte {
        Some(0) => {
            match bytes.pop() {
                Some(0) => return Some(Character::RecordSeperator),
                Some(1) => return Some(Character::ValueStart),
                Some(2) => return Some(Character::Regular(0)),
                Some(3) => return Some(Character::PointerStart),
                _ => panic!("Invalid character sequence")
            }
        },
        Some(ch) => return Some(Character::Regular(ch)),
        None => return None,
    }
}

fn get_value_vec(bytes: &mut Vec<u8>) -> Vec<u8> {
    let mut value_bytes = Vec::new();
    loop {
        match get_character(bytes) {
            Some(Character::Regular(byte)) => value_bytes.push(byte),
            Some(ch) => {
                // Put back what you have taken
                bytes.append(&mut ch.get_value());
                break; // we're done here.
            },
            None => break,
        }
    }
    return value_bytes;
}


impl Character {
    fn get_value(&self) -> Vec<u8> {
        let mut duple: Vec<u8> = vec!(0);
        match *self {
            Character::Regular(0)      => duple.push(2),
            Character::Regular(val)    => duple.push(val),
            Character::RecordSeperator => duple.push(0),
            Character::ValueStart      => duple.push(1),
            Character::PointerStart    => duple.push(3),
        }
        return duple;
    }
}

impl KVSet {
    pub fn new() -> KVSet {
        return KVSet {
            data: BTreeMap::new(),
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

    pub fn deserialize(bytes3: &mut Vec<u8>) -> Result<KVSet, CorruptDataError> {
        let mut datamap = BTreeMap::new();
        let mut pointermap = BTreeMap::new();
        let mut bytes = bytes3.clone();
        let length = bytes.len();

        // If the byte vector is empty, we're good.
        if (length == 0) {
            return Ok(KVSet {
                data: datamap,
                pointers: pointermap
            });
        }

        bytes.reverse();
        while(bytes.len() > 0) {
            let mut key_buffer = Vec::new();
            let mut val_buffer = Vec::new();
            let mut ptr_buffer = Vec::new();

            match get_character(&mut bytes) {
                Some(Character::ValueStart)      => val_buffer.append(&mut get_value_vec(&mut bytes)),
                Some(Character::PointerStart)    => ptr_buffer.append(&mut get_value_vec(&mut bytes)),
                Some(Character::RecordSeperator) => {
                    let key = String::from_utf8(key_buffer.clone()).unwrap(); // TODO: change unwrap to throw CorruptDataError
                    let val = String::from_utf8(val_buffer.clone()).unwrap(); // TODO: change unwrap to throw CorruptDataError

                    datamap.insert(key.clone(), val);

                    match ptr_buffer.len() {
                        0 => {}, // do nothing
                        8 => {
                            let block_number = unsafe { decode::<u64>(&mut ptr_buffer) };
                            pointermap.insert(key.clone(), *block_number.unwrap().0);
                        },
                        _ => panic!("Invalid pointer length")
                    }
                },
                Some(Character::Regular(ch)) => {
                    bytes.push(ch);
                    key_buffer.append(&mut get_value_vec(&mut bytes))
                },
                _ => panic!("Invalid pointer length")
            }
        }

        return Ok(KVSet {
            data: datamap,
            pointers: pointermap
        });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let null_bytes: Vec<u8> = vec!(0,2);

        for key in self.data.keys() {
            if (bytes.len() > 0) {
                bytes.append(&mut Character::RecordSeperator.get_value());
            }

            for byte in key.clone().into_bytes() {
                match byte {
                    0 => bytes.append(&mut null_bytes.clone()),
                    e => bytes.push(e),
                }
            }

            bytes.append(&mut Character::ValueStart.get_value());
            for byte in self.data.get(key).unwrap().clone().into_bytes() {
                match byte {
                    0 => bytes.append(&mut null_bytes.clone()),
                    e => bytes.push(e)
                }
            }

            if (self.pointers.contains_key(key)) {
                bytes.append(&mut Character::PointerStart.get_value());
                let mut vector = Vec::new();
                unsafe { encode(&self.pointers.get(key).unwrap().clone(), &mut vector); }
                for byte in vector {
                    bytes.push(byte);
                }
            }
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
