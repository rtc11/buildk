use std::{fs, io};
use std::path::{Path, PathBuf};

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::errors::CliError;

pub mod manifest;
pub mod config;
pub mod kotlin;
pub mod project;
pub mod build;
pub mod errors;

pub fn read_file<T: for<'a> Deserialize<'a>>(file: &Path) -> Result<T, Error> {
    let content = fs::read_to_string(file)?;
    toml::from_str(&content).with_context(|| format!("Failed to file as toml"))
}

pub fn write_file<T: Serialize>(file: &str, toml: &T) -> Result<(), Error> {
    let content = toml::to_string::<T>(&toml).with_context(|| "Failed to stringify toml Struct")?;
    fs::write(file, content).with_context(|| format!("Failed to create file {file}"))
}

fn buildk_home_dir() -> io::Result<PathBuf> {
    home::home_dir().map(|home| home.join(".buildk"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "$HOME env probably missing."))
}

pub struct LazyConfig {
    config: Option<Config>,
}

impl LazyConfig {
    pub fn new() -> Self { Self { config: None } }
    pub fn is_init(&self) -> bool { self.config.is_some() }
    pub fn get(&mut self) -> &Config { self.get_mut() }
    pub fn get_mut(&mut self) -> &mut Config {
        self.config.get_or_insert_with(|| match Config::default() {
            Ok(cfg) => cfg,
            Err(e) => exit_with_error(e.into())
        })
    }
}

fn exit_with_error(err: CliError) -> ! {
    let CliError { error, exit_code } = err;
    if let Some(error) = error {
        display_error(&error);
    }
    std::process::exit(exit_code)
}

fn display_error(err: &anyhow::Error) {
    eprintln!("{}", err)
}
