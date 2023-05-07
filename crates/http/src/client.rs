use std::fs::{create_dir_all, File};
use std::io;
use std::path::Path;
use anyhow::ensure;

use config::dependencies::dependency::Dependency;

#[derive(Default, Clone)]
pub struct Client;

impl Client {
    pub fn download(&mut self, dep: &Dependency) -> anyhow::Result<()> {
        create_dir_all(&dep.target_dir)?;

        download_file(&dep.url, &dep.target_dir, &dep.jar)?;
        download_file(&dep.url, &dep.target_dir, &dep.sources).ok();
        download_file(&dep.url, &dep.target_dir, &dep.module).ok();
        download_file(&dep.url, &dep.target_dir, &dep.pom)?;

        Ok(())
    }
}

fn download_file(url: &String, target_dir: &Path, filename: &String) -> anyhow::Result<()> {
    let mut destination = File::create(target_dir.join(filename))?;
    let url = &format!("{url}{filename}");
    let mut response = reqwest::blocking::get(url)?;
    ensure!(response.status().is_success());
    io::copy(&mut response, &mut destination)?;
    Ok(())
}