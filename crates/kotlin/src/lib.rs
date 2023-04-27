use std::path::PathBuf;
use std::sync::Mutex;

use cache::cache::Cache;
use config::config::Config;
use util::{BuildkResult, get_kotlin_home, PartialConclusion};
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

mod clean;
mod build;
mod test;
mod run;
mod release;

#[derive(Debug)]
pub struct Kotlin {
    pub version: String,
    cache: Mutex<Cache>,
    test_libs: Vec<PathBuf>,
}

impl Kotlin {
    pub fn new(
        config: &Config,
    ) -> BuildkResult<Kotlin> {
        let cache_location = config.cwd.join(config.manifest.build.output_cache());
        let kotlin_home = get_kotlin_home();
        let cache = Cache::load(&kotlin_home, &cache_location);

        let mut kotlinc = Kotlin {
            version: "unknown".to_string(),
            cache: Mutex::new(cache),
            test_libs: vec![
                kotlin_home.join("lib/kotlin-test-junit5.jar"),
                kotlin_home.join("lib/kotlin-test.jar"),
            ],
        };

        let mut runner = ProcessBuilder::new(kotlin_home.join("bin/kotlin"));
        runner.cwd(&config.cwd).arg("-version");

        let (verbose_version, _, _) = kotlinc.cache.lock().unwrap().cached_output(&runner, 0)?;

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
        let result = self.cache.lock().unwrap().cached_output(cmd, extra_fingerprint);
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
