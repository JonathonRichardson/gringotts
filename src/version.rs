#![allow(unused_parens)]

use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;

pub struct Version {
    major: u16,
    minor: u16,
    build: u16
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
