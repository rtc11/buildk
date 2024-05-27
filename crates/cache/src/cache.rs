use std::fmt::Display;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{PathBuf, Path};

use anyhow::Result;

use dependency::Package;
use util::buildk_output::{BuildkOutput, WithBKOutput};
use util::{PartialConclusion, paths};

use crate::{dependency_fingerprint, file_fingerprint};
use crate::data::CacheData;
use crate::output::Output;

#[derive(Clone)]
pub struct Cache {
    location: PathBuf,
    dirty: bool,
    data: CacheData,
}

#[derive(Debug)]
pub struct CacheResult {
    pub conclusion: PartialConclusion,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub status: i32,
}

pub trait Cacheable {
    type Item;

    fn cache(&mut self, cache: &mut Cache, item: Self::Item) -> Result<CacheResult>;
    fn fingerprint(&self, item: Self::Item) -> u64;
}

impl WithBKOutput for CacheResult {
    fn add_to_output<'a>(&'a self, out: &'a mut BuildkOutput) -> &'a mut BuildkOutput {
        out
            .conclude(self.conclusion.clone())
            .status(self.status)
            .stdout(self.stdout.clone().unwrap_or_default())
            .stderr(self.stderr.clone().unwrap_or_default())
    }
}

impl From<CacheResult> for BuildkOutput {
    fn from(value: CacheResult) -> Self {
        let conclusion = match value.stderr {
            Some(ref err) if !err.is_empty() => PartialConclusion::FAILED ,
            _ => value.conclusion,
        };

        BuildkOutput::new("temp")
            .conclude(conclusion)
            .status(value.status)
            .stdout(value.stdout.unwrap_or("".to_owned()))
            .stderr(value.stderr.unwrap_or("".to_owned()))
            .to_owned()
    }
}

impl Cache {
    pub fn load(cache_location: &Path) -> Cache {
        let location = cache_location.to_path_buf();
        let dirty = false;

        match Self::read(cache_location){
            Ok(data) => Cache { location, dirty, data },
            Err(_) => Cache { location, dirty, data: CacheData::default() }
        }
    }

    fn read(path: &Path) -> Result<CacheData> {
        let json = paths::read(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn insert(&mut self, key: u64, output: Output) {
        self.data.insert(key, output);
        self.dirty = true;
    }

    pub fn get(&self, key: &u64) -> &Output {
        self.data.get(key)
    }

    pub fn contains_key(&self, key: &u64) -> bool {
        self.data.contains_key(key)
    }

    pub fn cache_file(
        &mut self,
        file: &PathBuf,
    ) -> Result<PartialConclusion> {
        let key = file_fingerprint(file)?;
        match self.data.contains_key(&key) {
            true => Ok(PartialConclusion::CACHED),
            false => {
                let mut output = Output::default();
                output.set_action(file.to_string_lossy().to_string());
                output.set_success();
                self.data.insert(key, output);
                self.dirty = true;
                Ok(PartialConclusion::SUCCESS)
            }
        }
    }

    pub fn cache_dependency(
        &mut self,
        pkg: &Package,
    ) -> Result<PartialConclusion> {
        let key = dependency_fingerprint(pkg)?;
        match self.data.contains_key(&key) {
            true => Ok(PartialConclusion::CACHED),
            false => {
                let mut output = Output::default();
                output.set_action(pkg.location.to_string_lossy().to_string());
                output.set_stdout(pkg.classpath());
                self.data.insert(key, output);
                Ok(PartialConclusion::SUCCESS)
            }
        }
    }

    pub fn invalidate(&mut self) {
        self.dirty = false;
    }
}

impl Display for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cache: {}", self.location.display())?;
        write!(f, "Data: {}", self.data)
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        if !self.dirty { 
            return;
        }

        if let Some(path) = &self.location.parent() {
            if !path.exists(){
                if let Err(msg) = create_dir_all(path){
                    println!("failed to create missing directories {}. {msg}", path.display())
                }
            }
        }

        let file = File::create(&self.location);
        if let Ok(mut file) = file {
            if let Ok(text) = serde_json::to_string_pretty(&self.data) {
                if let Err(msg) = file.write_all(text.as_bytes()){
                    println!("Failed to write cache to {}. {msg}", self.location.display());
                }
            }
        }
    }
}

