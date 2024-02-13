use std::{path::{PathBuf, Path}, fmt::Display, hash::{Hash, Hasher}};

use anyhow::Result;
use cache::cache::{Cache, Cacheable, CacheResult};
use manifest::config::Config;
use util::{buildk_output::BuildkOutput, hasher::StableHasher, PartialConclusion, colorize::Colorize};

use crate::{ProcessBuilder, ProcessError, Process, try_from};

pub struct Kotlin<'a> {
    pub version: String,
    pub home: PathBuf,
    pub bin: PathBuf,
    config: &'a Config, 
}

impl <'a> Process<'a> for Kotlin<'a>  {
    type Item = Kotlin<'a>;

    fn new(config: &'a Config) -> Result<Self::Item> {
        let kotlin_home = match config.manifest.kotlin_home.as_ref() {
            Some(kotlin_home) => kotlin_home.clone(),
            None => {
                match option_env!("KOTLIN_HOME") {
                    Some(dir) => PathBuf::from(dir),
                    None => PathBuf::from("/usr/local/Cellar/kotlin/1.9.22/"),
                }
            }
        };

        let kotlin = Kotlin {
            config,
            version: version(config, &kotlin_home)?,
            bin: kotlin_home.join("bin"),
            home: kotlin_home.to_path_buf(),
        };
        
        Ok(kotlin)
    }
}

impl Kotlin<'_> {
    pub fn test_libs(&self) -> Vec<PathBuf> {
        vec![
            self.home.join("libexec/lib/kotlin-test-junit5.jar"),
            self.home.join("libexec/lib/kotlin-test.jar"),
        ]
    }

    pub fn builder(&self) -> KotlinBuilder {
        KotlinBuilder::new(self)
    }

    pub fn compiler(&self) -> PathBuf {
        self.bin.join("kotlinc")
    }


}

pub struct KotlinBuilder<'a>  {
    kotlin: &'a Kotlin<'a>,
    cache: Cache,
    cache_key: u64,
    process: ProcessBuilder,
}


/*
fn kotlinc_fingerprint(kotlin_bin: &Path) -> Result<u64> {
    let kotlinc = kotlin_bin.join(get_kotlinc());
    let mut hasher = StableHasher::default();
    hash(&mut hasher, &kotlinc)?;
    Ok(hasher.finish())
}
*/

impl <'a> KotlinBuilder<'a> {
    fn new(kotlin: &'a Kotlin<'a>) -> KotlinBuilder<'a> {
        KotlinBuilder {
            kotlin,
            cache: Cache::load(&kotlin.config.manifest.project.out.cache),
            cache_key: 0,
            process: ProcessBuilder::new(""),
        }
    }

    pub fn sources(&'a mut self, sources: Vec<&'a PathBuf>) -> &'a mut Self {
        for src in sources.iter() {
            self.process.sources(src);
        }
        self
    }

    pub fn source(&'a mut self, source: &'a PathBuf) -> &'a mut Self {
        self.process.sources(source);
        self
    }

    pub fn classpath(&'a mut self, classpath: Vec<&'a PathBuf>) -> &'a mut Self {
        self.process.classpaths(classpath);
        self
    }

    pub fn workdir(&'a mut self, workdir: &'a PathBuf) -> &'a mut Self {
        self.process.cwd(workdir);
        self
    }

    pub fn target(&'a mut self, target: &'a PathBuf) -> &'a mut Self {
        self.process.destination(target);
        self
    }

    pub fn cache_key(&mut self, key: u64) -> &mut Self {
        self.cache_key = key;
        self
    }

    pub fn run(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        self.process.program(self.kotlin.compiler());
        self.process.include_runtime();
        self.execute_with_cache(output, &self.process.clone()).to_owned()
    }

    pub fn compile(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        self.process.program(self.kotlin.compiler());
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

impl Cacheable for KotlinBuilder<'_> {
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

impl Display for Kotlin<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "kotlin.home", self.home.display())
    }
}

fn version(_config: &Config, kotlin_home: &Path) -> Result<String> {
    let mut runner = ProcessBuilder::new(kotlin_home.join("bin/kotlin"));
    runner.arg("-version");

    // TODO: fix this
    //let mut cache = Cache::load(&config.manifest.project.out.cache);
    /*
    let cache_res = cache.cache_command(&runner, 0)?;

    let version = cache_res
        .stdout
        .expect("kotlinc -version gave no stdout")
        .lines()
        .find(|l| l.starts_with("Kotlin version "))
        .map(|l| l.replace("Kotlin version ", ""))
        .ok_or_else(|| {
            anyhow::format_err!("`kotlinc -version` didnt have a line for `Kotlin version")
        })?;
    Ok(version)
*/
    Ok("1.9.22".into())
}

