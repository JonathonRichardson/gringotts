pub struct KeyChain {
    address: Vec<String>,
    final_key: String,
}

impl KeyChain {
    pub fn parse(input: &String) -> KeyChain {
        let mut pieces: Vec<String> = Vec::new();

        let mut cur_piece = String::new();
        let mut last_char: char = ' ';

        for ch in input.chars() {
            match ch {
                '/' if (last_char != '\\') => {
                    pieces.push(cur_piece.clone());
                    cur_piece.clear();
                },
                '/' if (last_char == '\\') => {
                    cur_piece.pop();
                    cur_piece.push('/');
                },
                c => cur_piece.push(c)
            }
            last_char = ch;
        }

        return KeyChain {
            address: pieces,
            final_key: cur_piece
        };
    }

    pub fn get_final_key(&self) -> String {
        return self.final_key.clone();
    }

    pub fn as_vec(&self) -> Vec<String> {
        return self.address.clone();
    }
}

impl IntoIterator for KeyChain {
    type Item = String;
    type IntoIter = ::std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.address.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let key = String::from("a/b/c");
        let keyref = KeyChain::parse(&key);

        assert_eq!(keyref.address, vec!("a", "b"));
        assert_eq!(keyref.get_final_key(), "c");
    }

    #[test]
    fn parsing_with_slashes() {
        let mut key = String::from("a");
        key.push('\\');
        key.push_str("/z/b/c");
        let keyref = KeyChain::parse(&key);

        assert_eq!(keyref.address, vec!("a/z", "b"));
        assert_eq!(keyref.get_final_key(), "c");
    }

    #[test]
    fn iterating_over_key() {
        let keychain = KeyChain::parse(&String::from("a/b/c"));
        let mut keys = Vec::new();

        let final_key = keychain.get_final_key();
        for key in keychain {
            keys.push(key);
        }

        assert_eq!(keys, vec!("a", "b"));
        assert_eq!(final_key, "c");
    }
}
