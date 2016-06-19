use dbfile::block::*;

pub const MAGIC_STRING: &'static str = "GringottsDBFile - https://github.com/JonathonRichardson/gringotts";

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

#[cfg(test)]
mod tests {
    use super::*;
    use dbfile::block::{HasSectionAddress};

    #[test]
    fn magic_string_length() {
        assert_eq!(MAGIC_STRING.len(), super::HeaderSection::MagicString.get_length());
    }
}
