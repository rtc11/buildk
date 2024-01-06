use anyhow::Context;
use std::fs::{create_dir_all, File};
use std::io;
use std::path::Path;

use manifest::dependencies::Dependency;
use manifest::repositories::Repository;

#[derive(Default, Clone)]
pub struct Client;

impl Client {
    pub fn download(&mut self, dep: &Dependency, repos: &[Repository]) -> anyhow::Result<()> {
        create_dir_all(&dep.target_dir)?;
        
        let mut success = false;

        for repo in repos {
            let url = url(&repo.url, &dep.path);
            let jar = download_file(&url, &dep.target_dir, &dep.jar)
                .with_context(|| format!("{} missing from {}", &dep.target_dir.join(&dep.jar).display(), repo.url));
            
            let pom = download_file(&url, &dep.target_dir, &dep.pom)
                .with_context(|| format!("{} missing from {}", &dep.target_dir.join(&dep.jar).display(), repo.url));
            
            download_file(&url, &dep.target_dir, &dep.sources).ok();
            download_file(&url, &dep.target_dir, &dep.module).ok();

            if jar.is_ok() && pom.is_ok() {
                success = true;
                break;
            }
        }

        if !success {
            anyhow::bail!("failed to download {}", dep);
        }

        Ok(())
    }
}

fn url(host: &str, path: &str) -> String {
    format!("{}/{}", host, path)
}

fn download_file(url: &String, target_dir: &Path, filename: &String) -> anyhow::Result<()> {
    let mut destination = File::create(target_dir.join(filename))?;
    let url = &format!("{url}{filename}");
    let mut response = reqwest::blocking::get(url)?;

    println!("{:?}", response);
    anyhow::ensure!(response.status().is_success(), "failed to download {}:{:?}", url, response);

    io::copy(&mut response, &mut destination)?;
    
    anyhow::ensure!(destination.metadata()?.len() == 0, "size of {} is 0", filename);

    Ok(())
}
