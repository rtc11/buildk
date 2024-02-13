use std::{path::PathBuf, hash::{Hash, Hasher}, ffi::OsStr};

use anyhow::Result;
use cache::cache::{Cache, Cacheable, CacheResult};
use manifest::config::Config;
use util::{hasher::StableHasher, PartialConclusion, buildk_output::BuildkOutput, colorize::Colorize};

use crate::{Process, ProcessBuilder, ProcessError, try_from};


pub struct Java<'a> {
    config: &'a Config, // TODO: lifetime for &'a Config
    pub version: String,
    pub home: PathBuf,
    pub bin: PathBuf,
}

impl <'a> Process<'a> for Java<'a> {
    type Item = Java<'a>;

    fn new(config: &'a Config) -> Result<Self::Item> {
        Ok(
            Java {
                config,
                version: "17.0.1".to_string(),
                home: PathBuf::from("/usr/local/Cellar/openjdk/17.0.1/"),
                bin: PathBuf::from("/usr/bin/"),
            }
        )
    }
}

pub struct JavaBuilder<'a> {
    java: &'a Java<'a>,
    cache: Cache,
    cache_key: u64,
    process: ProcessBuilder,
}

impl Java<'_> {
    pub fn builder(&self) -> JavaBuilder {
        JavaBuilder::new(self, self.config)
    }

    fn runtime(&self) -> PathBuf {
        self.bin.join("java")
    }

    fn compiler(&self) -> PathBuf {
        self.bin.join("javac")
    }
}

impl <'a> JavaBuilder<'a> {
    fn new(java: &'a Java, config: &'a Config) -> JavaBuilder<'a> {
        JavaBuilder {
            java,
            cache: Cache::load(&config.manifest.project.out.cache),
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
        self.execute_with_cache(output, &self.process.clone()).to_owned()
    }

    pub fn compile(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        self.process.program(self.java.compiler());
        self.execute_with_cache(output, &self.process.clone()).to_owned()
    }

    fn execute_with_cache(&mut self, output: &mut BuildkOutput, cmd: &ProcessBuilder) -> BuildkOutput {
        match self.cache(&mut self.cache.clone(), cmd.clone()) {
            Ok(cache_res) => output.apply(BuildkOutput::from(cache_res)).to_owned(), 
            Err(err) => {
                println!("\r{:#}", err.to_string().as_red());

                output
                    .conclude(PartialConclusion::FAILED)
                    .stderr(err.to_string())
                    .to_owned()
            }
        }
    }
}

impl <'a> Cacheable for JavaBuilder<'a> {
    type Item = ProcessBuilder;

    fn cache(&mut self, cache: &mut Cache, item: Self::Item) -> Result<cache::cache::CacheResult> {
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
                        status: output.code.unwrap_or(0)
                    })
            },
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

