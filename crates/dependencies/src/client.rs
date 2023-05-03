use std::fs::{create_dir_all, File};
use std::io;
use anyhow::Context;

use config::dependencies::dependency::Dependency;

#[derive(Default)]
pub struct Client;

impl Client {
    // todo: do I have to download all the contents, or only the one jar?
    pub fn download(&mut self, dep: &Dependency) -> anyhow::Result<()> {
        let dir = dep.path.parent().context("parent directory not found.")?;
        create_dir_all(dir)?;
        let mut destination = File::create(&dep.path)?;
        let mut response = reqwest::blocking::get(&dep.url)?;
        assert!(response.status().is_success());
        io::copy(&mut response, &mut destination)?;
        Ok(())
    }
}
