use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Mutex;

use cache::cache::Cache;
use config::config::Config;
use http::client::Client;
use util::{BuildkResult, get_kotlin_home, PartialConclusion};
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

mod clean;
mod build;
mod test;
mod run;
mod release;
mod fetch;
mod list;
mod help;

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Option {
    Clean,
    List,
    Fetch,
    BuildSrc,
    BuildTest,
    Test,
    Run,
    Release,
    Help,
}

impl Option {
    pub fn from(value: String) -> Vec<Option> {
        match value.as_str() {
            "clean" => vec![Option::Clean],
            "fetch" => vec![Option::Fetch],
            "build" => vec![Option::Fetch, Option::BuildSrc, Option::BuildTest],
            "test" => vec![Option::Fetch, Option::BuildSrc, Option::BuildTest, Option::Test],
            "run" => vec![Option::Fetch, Option::BuildSrc, Option::Run],
            "release" => vec![Option::Fetch, Option::BuildSrc, Option::Release],
            "list" => vec![Option::List],
            "help" => vec![Option::Help],
            _ => vec![]
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
            Option::List => "list",
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
    pub fn new(
        config: &Config,
    ) -> BuildkResult<Command> {
        let kotlin_home = get_kotlin_home();
        let cache = Cache::load(&kotlin_home, &config.manifest.project.out.cache);

        let mut kotlinc = Command {
            version: "unknown".to_string(),
            cache: Mutex::new(cache),
            test_libs: vec![
                kotlin_home.join("lib/kotlin-test-junit5.jar"),
                kotlin_home.join("lib/kotlin-test.jar"),
            ],
            client: Client::default(),
        };

        let mut runner = ProcessBuilder::new(kotlin_home.join("bin/kotlin"));
        runner.cwd(&config.manifest.project.path).arg("-version");

        let (verbose_version, _, _) = kotlinc.cache.lock().unwrap().cache_command(&runner, 0)?;

        let version = verbose_version.lines()
            .find(|l| l.starts_with("Kotlin version "))
            .map(|l| l.replace("Kotlin version ", ""))
            .ok_or_else(|| anyhow::format_err!("`kotlinc -version` didnt have a line for `Kotlin version `, got:\n{}", verbose_version))?;

        kotlinc.version = version;

        Ok(kotlinc)
    }

    fn execute(
        &self,
        output: &mut BuildkOutput,
        cmd: &ProcessBuilder,
        extra_fingerprint: u64,
    ) -> BuildkOutput {
        let result = self.cache.lock().unwrap().cache_command(cmd, extra_fingerprint);
        match result {
            Ok((stdout, stderr, conclusion)) => output
                .conclude(conclusion)
                .stdout(stdout)
                .stderr(stderr)
                .clone(),
            Err(err) => output
                .conclude(PartialConclusion::FAILED)
                .stderr(err.to_string())
                .clone()
        }
    }

    fn invalidate_cache(&mut self) {
        self.cache.lock().unwrap().invalidate()
    }
}
