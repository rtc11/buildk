use std::{ffi::OsStr, hash::{Hash, Hasher}, path::PathBuf};

use anyhow::Result;

use cache::cache::{Cache, Cacheable, CacheResult};
use manifest::config::BuildK;
use util::{buildk_output::BuildkOutput, hasher::StableHasher, PartialConclusion};
use util::buildk_output::WithBKOutput;

use crate::{Process, ProcessBuilder, ProcessError, try_from};

pub struct Java<'a> {
    buildk: &'a BuildK,
    pub version: String,
    pub home: PathBuf,
    pub bin: PathBuf,
}

impl<'a> Process<'a> for Java<'a> {
    type Item = Java<'a>;

    fn new(buildk: &'a BuildK) -> Result<Self::Item> {
        Ok(
            Java {
                buildk,
                version: "17.0.1".to_string(), // TODO: add version to buildk.toml
                home: PathBuf::from("/usr/local/Cellar/openjdk/17.0.1/"),
                bin: PathBuf::from("/usr/bin/"),
            }
        )
    }
}

impl<'a> Java<'a> {
    pub fn builder(&self) -> JavaBuilder {
        JavaBuilder::new(self)
    }

    fn runtime(&self) -> PathBuf {
        self.bin.join("java")
    }

    fn compiler(&self) -> PathBuf {
        self.bin.join("javac")
    }
}

pub struct JavaBuilder<'a> {
    java: &'a Java<'a>,
    cache: Cache,
    cache_key: u64,
    process: ProcessBuilder,
}

impl<'a> JavaBuilder<'a> {
    fn new(java: &'a Java) -> JavaBuilder<'a> {
        let manifest = java.buildk.clone().manifest
            .expect("no buildk.toml found.");

        JavaBuilder {
            java,
            cache: Cache::load(&manifest.project.out_paths().cache),
            cache_key: 0,
            process: ProcessBuilder::new(""),
        }
    }

    pub fn main(&mut self, main: String) -> &mut Self {
        self.process.sources(&main);
        self
    }

    pub fn classpath(&mut self, classpath: Vec<&PathBuf>) -> &mut Self {
        self.process.classpaths(classpath);
        self
    }

    pub fn workdir(&mut self, workdir: &PathBuf) -> &mut Self {
        self.process.cwd(workdir);
        self
    }

    pub fn jar(&mut self, jar: &PathBuf) -> &mut Self {
        self.process.args.push(jar.into());
        self
    }

    pub fn test_report(&mut self, report_dir: &PathBuf) -> &mut Self {
        self.process.test_report(report_dir);
        self
    }

    pub fn args<T: AsRef<OsStr>>(&mut self, args: &[T]) -> &mut Self {
        self.process.args(args);
        self
    }

    pub fn run(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        self.process.program(self.java.runtime());
        let mut cache = self.cache.clone();
        match self.cache(&mut cache, self.process.clone()) {
            Ok(result) => result.add_to_output(output).to_owned(),
            Err(err) => {
                //println!("\r{:#}", err.to_string().as_red());
                output.conclude(PartialConclusion::FAILED).stderr(err.to_string()).to_owned()
            }
        }
    }

    pub fn compile(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        self.process.program(self.java.compiler());
        let mut cache = self.cache.clone();
        match self.cache(&mut cache, self.process.clone()) {
            Ok(result) => result.add_to_output(output).to_owned(),
            Err(err) => {
                //println!("\r{:#}", err.to_string().as_red());
                output.conclude(PartialConclusion::FAILED).stderr(err.to_string()).to_owned()
            }
        }
    }
}

pub trait ProcessCacher {
    fn cache(&mut self, item: ProcessBuilder) -> Result<CacheResult>;
    fn fingerprint(&self, item: ProcessBuilder) -> u64;
}

impl<'a> Cacheable for JavaBuilder<'a> {
    type Item = ProcessBuilder;

    fn cache(&mut self, cache: &mut Cache, item: Self::Item) -> Result<CacheResult> {
        let key = self.fingerprint(item.clone());
        let partial_conclusion = match cache.contains_key(&key) {
            true => PartialConclusion::CACHED,
            false => {
                let output = item.output()?;
                cache.insert(key, try_from(&item, output)?);
                PartialConclusion::SUCCESS
            }
        };

        let output = cache.get(&key);
        match output.success {
            true => {
                Ok(CacheResult {
                    conclusion: partial_conclusion,
                    stdout: Some(output.stdout.clone()),
                    stderr: Some(output.stderr.clone()),
                    status: output.code.unwrap_or(0),
                })
            }
            false => {
                Err(ProcessError::new_with_raw_output(
                    &format!("process didn't exit successfully (cache): {item}"),
                    output.code,
                    &output.status,
                    Some(output.stdout.as_ref()),
                    Some(output.stderr.as_ref()),
                ).into())
            }
        }
    }

    fn fingerprint(&self, item: Self::Item) -> u64 {
        let mut hasher = StableHasher::default();
        self.cache_key.hash(&mut hasher);
        item.get_args().for_each(|arg| arg.hash(&mut hasher));
        let mut env = item.get_envs().iter().collect::<Vec<_>>();
        env.sort_unstable();
        env.hash(&mut hasher);
        hasher.finish()
    }
}

