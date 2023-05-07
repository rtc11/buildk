use std::fs::{create_dir_all, File};
use std::path::{Path, PathBuf};
use config::dependencies::dependency::Dependency;

use util::{BuildkResult, PartialConclusion, paths};
use util::process_builder::ProcessBuilder;
use util::process_error::ProcessError;

use crate::{dependency_fingerprint, kotlinc_fingerprint, kt_fingerprint, process_fingerprint};
use crate::data::CacheData;
use crate::output::Output;

pub struct Cache {
    location: PathBuf,
    dirty: bool,
    data: CacheData,
}

impl Cache {
    pub fn load(kotlin_home: &Path, cache_location: &Path) -> Cache {
        let kotlin_bin = kotlin_home.join("bin");

        match kotlinc_fingerprint(&kotlin_bin) {
            Ok(fingerprint) => {
                let empty = CacheData::empty(fingerprint);
                let mut dirty = true;

                let data = match read(cache_location) {
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

                fn read(path: &Path) -> BuildkResult<CacheData> {
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
    ) -> BuildkResult<(String, String, PartialConclusion)> {
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
            true => Ok((output.stdout.clone(), output.stderr.clone(), partial_conclusion)),
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
    ) -> BuildkResult<PartialConclusion> {
        let key = kt_fingerprint(file)?;
        match self.data.contains_key(&key) {
            true => Ok(PartialConclusion::CACHED),
            false => {
                let mut output = Output::default();
                output.set_action(file.to_string_lossy().to_string());
                self.data.insert(key, output);
                self.dirty = true;
                Ok(PartialConclusion::SUCCESS)
            }
        }
    }

    pub fn cache_dependency(
        &mut self,
        dep: &Dependency,
    ) -> BuildkResult<PartialConclusion> {
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
        if !self.dirty { return; }

        if let Some(path) = &self.location.parent() {
            if !path.exists() {
                if let Err(msg) = create_dir_all(path) {
                    println!("failed to create missing director(y/ies) {}. {msg}", path.display())
                }
            }
        }

        match File::create(&self.location) {
            Err(e) => println!("failed to create cache file {}: {}", self.location.display(), e),
            Ok(file) => if let Err(e) = serde_json::to_writer_pretty(&file, &self.data) {
                println!("failed to update kotlinc info cache: {e}")
            }
        }
    }
}
