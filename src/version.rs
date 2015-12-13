#![allow(unused_parens)]

use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;

pub struct Version {
    major: u16,
    minor: u16,
    build: u16
}

impl Version {
    pub fn from_bytes(bytes: &Vec<u8>) -> Version {
        if (bytes.len() != 6) {
            panic!("Version Strings should be 8bytes long.")
        }

        return Version {
            major: (bytes[0] as u16) | ((bytes[1] as u16) << 8),
            minor: (bytes[2] as u16) | ((bytes[3] as u16) << 8),
            build: (bytes[4] as u16) | ((bytes[5] as u16) << 8),
        }
    }
}

impl PartialEq<Version> for Version {
    fn eq(&self, other: &Version) -> bool {
        // Check in reverse order, since that's more likely to differ
        if ( self.build != other.build ) {
            return false;
        }

        if ( self.minor != other.minor ) {
            return false;
        }

        if ( self.major != other.major ) {
            return false;
        }

        return true;
    }
}

impl PartialOrd<Version> for Version {
    // Self is ___ than/to Other...
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        let mut ordering: Ordering;

        ordering = self.major.partial_cmp(&other.major).unwrap();
        if (ordering != Ordering::Equal) {
            return Some(ordering);
        }

        ordering = self.minor.partial_cmp(&other.minor).unwrap();
        if (ordering != Ordering::Equal) {
            return Some(ordering);
        }

        ordering = self.build.partial_cmp(&other.build).unwrap();
        if (ordering != Ordering::Equal) {
            return Some(ordering);
        }

        return Some(Ordering::Equal);
    }
}
