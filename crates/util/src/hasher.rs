#![allow(deprecated)]

use std::hash::{Hasher, SipHasher};

pub struct StableHasher(SipHasher);

impl Default for StableHasher {
    fn default() -> Self {
        StableHasher(SipHasher::new())
    }
}

impl Hasher for StableHasher {
    fn finish(&self) -> u64 {
        self.0.finish()
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }
}
