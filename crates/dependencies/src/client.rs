#![allow(dead_code)]

use std::fs::{create_dir_all, File};
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::bail;
use regex::Regex;

use config::config::Config;

// TODO: use fingerprint to retrieve cache?
#[derive(Debug)]
pub struct DependencyInfo {
    pub path: PathBuf,
    pub filename: String,
    pub url: String,
    pub name: String,
    pub version: String,
}

#[derive(Default)]
struct DependencyCache {
    dependencies: Vec<DependencyInfo>,
}

pub struct Client {
    cache_location: PathBuf,
    cache: Mutex<DependencyCache>,
}

impl DependencyInfo {
    pub fn is_cached(&self) -> bool {
        self.path.join(&self.filename).is_file()
    }
}

impl Client {
    pub fn new(config: &Config) -> Self {
        Client {
            cache_location: config.home.join("cache"),
            cache: Mutex::default(),
        }
    }

    /// [name] "org.apache.kafka.kafka-clients" [version] "3.4.0"
    pub fn dependency_info(&mut self, name: &str, version: &str) -> anyhow::Result<DependencyInfo> {
        let after_last_slash = Regex::new(r"([^/]+)$").unwrap();
        let dependency = name.replace('.', "/");

        match after_last_slash.find(&dependency) {
            None => bail!("artifact not found for dependency"),
            Some(artifact_name) => {
                match dependency.split('/').map(PathBuf::from).reduce(|a, b| a.join(b)) {
                    None => bail!("relative path for dependency not deduced"),
                    Some(relative_path) => {
                        let jar = format!("{}-{version}.jar", artifact_name.as_str());
                        Ok(DependencyInfo {
                            url: format!("https://repo1.maven.org/maven2/{dependency}/{version}/{jar}"),
                            filename: jar,
                            path: self.cache_location.join(relative_path),
                            name: artifact_name.as_str().to_string(),
                            version: version.to_string(),
                        })
                    }
                }
            }
        }
    }

    // todo: do I have to download all the contents, or only the one jar?
    pub fn download(&mut self, dep: &DependencyInfo) -> anyhow::Result<()> {
        let filepath = dep.path.join(&dep.filename);
        create_dir_all(&dep.path)?;
        let mut destination = File::create(filepath)?;
        let response = reqwest::blocking::get(&dep.url)?;
        let content = response.text()?;
        io::copy(&mut content.as_bytes(), &mut destination)?;
        Ok(())
    }
}
