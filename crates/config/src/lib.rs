extern crate core;

use std::fs;
use std::path::Path;

use anyhow::Context;

pub mod manifest;
pub mod config;
pub mod project;
pub mod dependencies;
pub mod modules;
pub(crate) mod section;
pub mod classpath;

pub fn read_file(file: &Path) -> anyhow::Result<String> {
    fs::read_to_string(file).context(format!("File not found: {}", file.display()))
}

pub mod buildk {
    use std::io;
    use std::path::PathBuf;

    use anyhow::Context;

    pub fn home_dir() -> anyhow::Result<PathBuf> {
        home::home_dir().map(|home| home.join(".buildk"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "$HOME env probably missing."))
            .with_context(|| "buildk could not find its home directory")
    }
}
