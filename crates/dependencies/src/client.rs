#![allow(dead_code)]

use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::bail;
use regex::Regex;
use tempfile::Builder;

use config::config::Config;

// TODO: use fingerprint to retrieve cache?
#[derive(Debug)]
pub struct DependencyInfo {
    path: PathBuf,
    url: String,
    name: String,
    version: String,
}

#[derive(Default)]
struct DependencyCache {
    dependencies: Vec<DependencyInfo>,
}

pub struct Client {
    cache_location: PathBuf,
    cache: Mutex<DependencyCache>,
}

impl Client {
    pub fn new(config: &Config) -> Self {
        Client {
            cache_location: config.home.join("cache"),
            cache: Mutex::default(),
        }
    }

    /// artifact: e.g. `org/apache/kafka/kafka-clients`
    /// version: e.g. `3.4.0`
    pub fn download_info(&mut self, dependency: &str, version: &str) -> anyhow::Result<DependencyInfo> {
        let after_last_slash = Regex::new(r"([^/]+)$").unwrap();
        let dependency = dependency.replace('.', "/");

        match after_last_slash.find(&dependency) {
            None => bail!("artifact not found for dependency"),
            Some(artifact_name) => {
                match dependency.split('/').map(PathBuf::from).reduce(|a, b| a.join(b)) {
                    None => bail!("relative path for dependency not deduced"),
                    Some(relative_path) => {
                        // println!("relative path for dependency: {}", relative_path.display());
                        let jar = format!("{}-{version}.jar", artifact_name.as_str());
                        Ok(DependencyInfo {
                            url: format!("https://repo1.maven.org/maven2/{dependency}/{version}/{jar}"),
                            path: self.cache_location.join(relative_path),
                            name: artifact_name.as_str().to_string(),
                            version: version.to_string(),
                        })
                    }
                }
            }
        }
    }

    pub async fn download(&mut self, dep: DependencyInfo) -> anyhow::Result<()> {
        let temp_dir = Builder::new().prefix(&dep.name).tempdir()?;
        let response = reqwest::get(dep.url).await?;

        let mut dest = {
            let fname = response.url()
                .path_segments()
                .and_then(|segments| segments.last())
                .and_then(|name| if name.is_empty() { None } else { Some(name) })
                .unwrap_or("temp_dep.bin");

            println!("file to download: '{}'", fname);
            let fname = temp_dir.path().join(fname);
            println!("will be located under: '{:?}'", fname);
            File::create(fname)?
        };
        let content = response.text().await?;
        io::copy(&mut content.as_bytes(), &mut dest)?;
        Ok(())
    }
}
