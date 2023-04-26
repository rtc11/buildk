extern crate core;

use std::{fs, io};
use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

pub mod manifest;
pub mod config;
pub mod kotlin;
pub mod project;
pub mod build;

pub fn read_file<T: for<'a> Deserialize<'a>>(file: &Path) -> Result<T, Error> {
    let content = fs::read_to_string(file)?;
    toml::from_str(&content).with_context(|| "Failed to file as toml")
}

pub fn write_file<T: Serialize>(file: &str, toml: &T) -> Result<(), Error> {
    let content = toml::to_string::<T>(toml).with_context(|| "Failed to stringify toml Struct")?;
    fs::write(file, content).with_context(|| format!("Failed to create file {file}"))
}

fn buildk_home_dir() -> io::Result<PathBuf> {
    home::home_dir().map(|home| home.join(".buildk"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "$HOME env probably missing."))
}
