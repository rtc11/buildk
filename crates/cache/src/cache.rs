use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{PathBuf, Path};

use anyhow::Result;
use manifest::dependencies::Dependency;

use util::{PartialConclusion, paths};
use util::process_builder::ProcessBuilder;
use util::process_error::ProcessError;

use crate::{dependency_fingerprint, kotlinc_fingerprint, file_fingerprint, process_fingerprint};
use crate::data::CacheData;
use crate::output::Output;

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

impl Cache {
    pub fn load(kotlin_home: &Path, cache_location: &Path) -> Cache {
        let kotlin_bin = kotlin_home.join("bin");

        match kotlinc_fingerprint(&kotlin_bin){
            Ok(fingerprint) => {
                let empty = CacheData::empty(fingerprint);
                let mut dirty = true;

                let data = match read(cache_location){
                    Ok(data) => {
                        if data.fingerprint() == fingerprint {
                            dirty = false;
                            data
                        } else {
                            empty
                        }
                    }
                    Err(_) => empty
                };
                let location = cache_location.to_path_buf();
                return Cache { location, dirty, data };

                fn read(path: &Path) -> Result<CacheData> {
                    let json = paths::read(path)?;
                    Ok(serde_json::from_str(&json)?)
                }
            }
            Err(e) => {
                eprintln!("no cache found {e}");

                Cache {
                    location: cache_location.to_path_buf(),
                    dirty: false,
                    data: CacheData::default(),
                }
            }
        }
    }

    pub fn cache_command(
        &mut self,
        cmd: &ProcessBuilder,
        extra_fingerprint: u64,
    ) -> Result<CacheResult> {
        let key = process_fingerprint(cmd, extra_fingerprint);
        let partial_conclusion = match self.data.contains_key(&key) {
            true => PartialConclusion::CACHED,
            false => {
                let output = cmd.output()?;
                self.data.insert(key, Output::try_from(cmd, output)?);
                self.dirty = true;
                PartialConclusion::SUCCESS
            }
        };

        let output = self.data.get(&key);

        match output.success {
            true => {
                Ok(CacheResult {
                    conclusion: partial_conclusion,
                    stdout: Some(output.stdout.clone()),
                    stderr: Some(output.stderr.clone()),
                    status: output.code.unwrap_or(0)
                })
            },
            false => Err(ProcessError::new_with_raw_output(
                &format!("process didn't exit successfully (cache): {cmd}"),
                output.code,
                &output.status,
                Some(output.stdout.as_ref()),
                Some(output.stderr.as_ref()),
            ).into())
        }
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
        dep: &Dependency,
    ) -> Result<PartialConclusion> {
        let key = dependency_fingerprint(dep)?;
        match self.data.contains_key(&key) {
            true => Ok(PartialConclusion::CACHED),
            false => {
                let mut output = Output::default();
                output.set_action(dep.target_dir.to_string_lossy().to_string());
                output.set_stdout(dep.classpath());
                self.data.insert(key, output);
                Ok(PartialConclusion::SUCCESS)
            }
        }
    }

    pub fn invalidate(&mut self) {
        self.dirty = false;
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
                    println!("failed to create missing director(y/ies) {}. {msg}", path.display())
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

