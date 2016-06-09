use std::collections::BTreeMap;
use std::str;
use error::*;

pub struct KVPair {
    key: String,
    value: String,
}

pub struct KVSet {
    pairs: BTreeMap<String, String>
}

impl KVSet {
    pub fn new() -> KVSet {
        return KVSet {
            pairs: BTreeMap::new()
        }
    }

    /// Adds a new key/value pair to the KVSet.  The return value will be a String, if there was
    /// a previous value and this is therefore an update, or None, if this was a true insert.
    pub fn put(&mut self, key: String, value: String) -> Option<String> {
        return self.pairs.insert(key, value); // this returns the old value or None
    }

    pub fn get(&self, key: String) -> Option<&String> {
        return self.pairs.get(&key);
    }

    pub fn deserialize(bytes: &mut Vec<u8>) -> Result<KVSet, CorruptDataError> {
        let mut map = BTreeMap::new();
        let length = bytes.len();

        // If the byte vector is empty, we're good.
        if (length == 0) {
            return Ok(KVSet {
                pairs: map,
            });
        }

        let mut utf8_buffer = Vec::new();
        let mut escape_buffer = Vec::new();
        let mut cur_key = String::new();

        for i in 0..bytes.len() {
            if (escape_buffer.len() == 0) {
                if (bytes[i] != 0) {
                    utf8_buffer.push(bytes[i]);
                }
                else {
                    escape_buffer.push(bytes[i]);
                }
            }
            else {
                // We're an escape character
                match bytes[i] {
                    0 => {
                        let val = String::from_utf8(utf8_buffer.clone()).unwrap(); // TODO: change unwrap to throw CorruptDataError
                        map.insert(cur_key.clone(), val);
                        utf8_buffer.clear();
                    },
                    1 => {
                        cur_key = String::from_utf8(utf8_buffer.clone()).unwrap(); // TODO: change unwrap to throw CorruptDataError
                        utf8_buffer.clear();
                    },
                    2 => utf8_buffer.push(0), // escaped null.
                    _ => return Err(CorruptDataError::new("Unrecognized escape command sequence")),
                }

                // Reset the buffers
                utf8_buffer.clear();
                escape_buffer.clear();
            }
        }

        return Ok(KVSet {
            pairs: map
        });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for key in self.pairs.keys() {
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

            for byte in self.pairs.get(key).unwrap().clone().into_bytes() {
                if (byte == 0) {
                    bytes.push(0);
                    bytes.push(2);
                }
                else {
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
