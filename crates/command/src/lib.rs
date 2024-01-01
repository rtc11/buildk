use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Mutex;

use cache::cache::Cache;
use ::config::config::Config;
use http::client::Client;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::{get_kotlin_home, BuildkResult, PartialConclusion};

mod build;
mod clean;
mod config;
mod deps;
mod fetch;
mod help;
mod build_tree;
mod release;
mod run;
mod test;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Option {
    Clean,
    Deps,
    Fetch,
    BuildSrc,
    BuildTest,
    Test,
    Run,
    Release,
    BuildTree,
    Config,
    Help,
}

impl Option {
    pub fn from(value: String) -> Vec<Option> {
        match value.as_str() {
            "clean" => vec![Option::Clean],
            "fetch" => vec![Option::Fetch],
            "build" => vec![Option::Fetch, Option::BuildSrc, Option::BuildTest],
            "test" => vec![
                Option::Fetch,
                Option::BuildSrc,
                Option::BuildTest,
                Option::Test,
            ],
            "run" => vec![Option::Fetch, Option::BuildSrc, Option::Run],
            "release" => vec![Option::Fetch, Option::BuildSrc, Option::Release],
            "deps" => vec![Option::Deps],
            "tree" => vec![Option::BuildTree],
            "config" => vec![Option::Config],
            "help" => vec![Option::Help],
            _ => vec![],
        }
    }
}

impl Display for Option {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            Option::Clean => "clean",
            Option::BuildSrc => "build src",
            Option::BuildTest => "build test",
            Option::Fetch => "fetch",
            Option::Test => "test",
            Option::Run => "run",
            Option::Release => "release",
            Option::Deps => "deps",
            Option::BuildTree => "tree",
            Option::Config => "config",
            Option::Help => "help",
        };

        write!(f, "{:<12}", display)
    }
}

pub struct Command {
    pub version: String,
    pub cache: Mutex<Cache>,
    test_libs: Vec<PathBuf>,
    pub client: Client,
}

impl Command {
    pub fn new(config: &Config) -> BuildkResult<Command> {
        let kotlin_home = get_kotlin_home();
        let cache = Cache::load(&kotlin_home, &config.manifest.project.out.cache);

        let mut kotlinc = Command {
            version: "unknown".to_string(),
            cache: Mutex::new(cache),
            test_libs: vec![
                kotlin_home.join("libexec/lib/kotlin-test-junit5.jar"),
                kotlin_home.join("libexec/lib/kotlin-test.jar"),
            ],
            client: Client
        };

        let mut runner = ProcessBuilder::new(kotlin_home.join("bin/kotlin"));
        runner.cwd(&config.manifest.project.path).arg("-version");

        let cache_res = kotlinc.cache.lock().unwrap().cache_command(&runner, 0)?;
        let version = cache_res
            .stdout
            .expect("kotlinc -version gave no stdout")
            .lines()
            .find(|l| l.starts_with("Kotlin version "))
            .map(|l| l.replace("Kotlin version ", ""))
            .ok_or_else(|| {
                anyhow::format_err!("`kotlinc -version` didnt have a line for `Kotlin version")
            })?;

        kotlinc.version = version;

        Ok(kotlinc)
    }

    fn execute(
        &self,
        output: &mut BuildkOutput,
        cmd: &ProcessBuilder,
        extra_fingerprint: u64,
    ) -> BuildkOutput {
        match self
            .cache
            .lock()
            .unwrap()
            .cache_command(cmd, extra_fingerprint)
        {
            Ok(cache_res) => {
                output
                    .conclude(cache_res.conclusion)
                    .status(cache_res.status);
                if let Some(stdout) = cache_res.stdout {
                    output.stdout(stdout);
                }
                if let Some(stderr) = cache_res.stderr {
                    output.stderr(stderr);
                }
                output.clone()
            }
            Err(err) => output
                .conclude(PartialConclusion::FAILED)
                .stderr(err.to_string())
                .clone(),
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache.lock().unwrap().invalidate()
    }
}
