use std::process::Command;

use anyhow::{Context, Result};
use fs::toml::Config;
use crate::log_result;

pub(crate) fn run(config: &Config) -> Result<String> {
    print!("▸ {:<7}", "run test");

    let classpath = [
        config.src_dir(),
        config.test_dir()
    ].join(":");

    let output = Command::new("java")
        .current_dir(&config.project.dir)
        .args(["-jar", "libs/junit-platform-console-standalone-1.9.2.jar"])
        .args(["-cp", &classpath])
        // .args(["--select-package", "no.tordly.test"])
        .args(["--select-class", "PrefixTest"])
        .output()
        .with_context(|| "failed to run tests")?;

    log_result(output)
}
