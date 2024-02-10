#![allow(dead_code)]
use std::fmt::Display;

use async_std::path::PathBuf;

#[derive(Clone)]
pub struct Module {
    name: String,
    path: PathBuf,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<26}{} ({})", "module:", self.path.display(), self.name)
    }
}

