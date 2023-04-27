use std::fs::File;
use std::path::{Path, PathBuf};

use util::{BuildkResult, PartialConclusion, paths};
use util::process_builder::ProcessBuilder;
use util::process_error::ProcessError;

use crate::{kotlinc_fingerprint, process_fingerprint};
use crate::data::CacheData;

#[derive(Debug)]
pub struct Cache {
    location: PathBuf,
    dirty: bool,
    data: CacheData,
}

impl Cache {
    #[cfg(not(debug_assertions))]
    fn kotlin_bin_path(kotlin_bin: &Path) -> PathBuf { PathBuf::from(kotlin_bin) }

    #[cfg(debug_assertions)]
    fn kotlin_bin_path(_kotlin_bin: &Path) -> PathBuf {
        PathBuf::from("kotlinc/bin")
    }

    pub fn load(kotlin_bin: &Path, cache_location: &Path) -> Cache {
        // temprary fix for relative path in dev
        let kotlin_bin = Self::kotlin_bin_path(kotlin_bin);

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

    pub fn cached_output(
        &mut self,
        cmd: &ProcessBuilder,
        extra_fingerprint: u64,
    ) -> BuildkResult<(String, String, PartialConclusion)> {
        let key = process_fingerprint(cmd, extra_fingerprint);
        let partial_conclusion = match self.data.contains_key(&key) {
            true => PartialConclusion::CACHED,
            false => {
                let output = cmd.output()?;
                self.data.insert(key, output.try_into()?);
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

    pub fn invalidate(&mut self) {
        self.dirty = false;
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        if !self.dirty { return; }

        match File::create(&self.location) {
            Err(e) => println!("failed to create cache file: {e}"),
            Ok(file) => if let Err(e) = serde_json::to_writer_pretty(&file, &self.data) {
                println!("failed to update kotlinc info cache: {e}")
            }
        }
    }
}
