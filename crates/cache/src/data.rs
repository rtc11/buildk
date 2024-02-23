use std::{collections::HashMap, fmt::Display};

use serde_derive::{Deserialize, Serialize};

use crate::output::Output;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct CacheData {
    kotlinc_fingerprint: u64,
    outputs: HashMap<u64, Output>,
    successes: HashMap<u64, bool>,
}

impl CacheData {
    pub fn contains_key(&self, key: &u64) -> bool {
        self.outputs.contains_key(key)
    }

    pub fn get(&self, key: &u64) -> &Output {
        &self.outputs[key]
    }

    pub fn insert(&mut self, key: u64, value: Output) {
        self.outputs.insert(key, value);
    }
}

impl Display for CacheData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.outputs.iter() {
            write!(f, "{}: {}", key, value)?;
        }

        write!(f, "")
    }
}
