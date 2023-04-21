use std::process::Command;

use anyhow::{Context, Result};
use console::PartialConclusion;
use fs::toml::Config;
use crate::log_result;

pub(crate) fn run(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "run", "test");

    let classpath = [
        config.build.output_src(),
        config.build.output_test(),
    ].join(":");

    let output = Command::new("java")
        .current_dir(&config.project.dir)
        .args(["-jar", "libs/junit-platform-console-standalone-1.9.2.jar"])
        .args(["-cp", &classpath])
        // .args(["--select-package", "no.tordly.test"])
        .args(["--select-class", "PrefixTest"])
        .output()
        .with_context(|| PartialConclusion::FAILED)?;

    log_result(output)
}
