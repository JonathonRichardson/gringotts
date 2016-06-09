use std::collections::LinkedList;
use std::str;
use error::*;

pub struct KVPair {
    key: String,
    value: String,
}

pub struct KVSet {
    pairs: LinkedList<KVPair>
}

impl KVSet {
    pub fn new() -> KVSet {
        return KVSet {
            pairs: LinkedList::new()
        }
    }

    /// Adds a new key/value pair to the KVSet.  The return value will be a String, if there was
    /// a previous value and this is therefore an update, or None, if this was a true insert.
    /*
    pub fn put(key: String, value: String) -> Option<String> {

    }

    pub fn get(key: String) -> Option<String> {

    }
    */

    pub fn deserialize(bytes: &mut Vec<u8>) -> Result<KVSet, CorruptDataError> {
        let mut list = LinkedList::new();
        let length = bytes.len();

        // If the byte vector is empty, we're good.
        if (length == 0) {
            return Ok(KVSet {
                pairs: list,
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

                        list.push_back(KVPair {
                            key: cur_key.clone(),
                            value: val,
                        });
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
            pairs: list
        });
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for kvpair in self.pairs.iter() {
            if (bytes.len() > 0) {
                bytes.push(0);
                bytes.push(0);
            }

            for byte in kvpair.key.clone().into_bytes() {
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

            for byte in kvpair.value.clone().into_bytes() {
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
    use std::collections::LinkedList;


    #[test]
    fn basic_serialization() {
        let mut list: LinkedList<KVPair> = LinkedList::new();

        list.push_back(KVPair {
            key: String::from("yes"),
            value: String::from("no")
        });

        list.push_back(KVPair {
            key: String::from("hello"),
            value: String::from("goodbye")
        });

        let keyset = KVSet {
            pairs: list
        };

        assert_eq!(keyset.serialize(), vec!(
            121, 101, 115,
            0, 1,
            110, 111,
            0, 0,
            104, 101, 108, 108, 111,
            0, 1,
            103, 111, 111, 100, 98, 121, 101
         ));
    }
}
