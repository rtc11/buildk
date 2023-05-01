extern crate core;

use std::fs;
use std::path::Path;

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

pub mod manifest;
pub mod config;
pub mod project;
pub mod build;
pub mod dependencies;
pub mod module;

pub fn read_file(file: &Path) -> anyhow::Result<String> {
    fs::read_to_string(file).context(format!("File not found: {}", file.display()))
}

pub fn toml<T: for<'a> Deserialize<'a>>(content: &str) -> anyhow::Result<T> {
    toml::from_str(content).with_context(|| format!("Failed to parse string to toml:\n{}", content))
}

pub fn write_file<T: Serialize>(file: &str, toml: &T) -> Result<(), Error> {
    let content = toml_edit::ser::to_string::<T>(toml).with_context(|| "Failed to stringify toml Struct")?;
    fs::write(file, content).with_context(|| format!("Failed to create file {file}"))
}

mod buildk {
    use std::io;
    use std::path::PathBuf;

    use anyhow::Context;

    pub fn home_dir() -> anyhow::Result<PathBuf> {
        home::home_dir().map(|home| home.join(".buildk"))
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "$HOME env probably missing."))
            .with_context(|| "buildk could not find its home directory")
    }
}
