use std::process::Command;

use anyhow::{Context, Result};

use console::PartialConclusion;
use fs::toml::Config;

use crate::{KOTLINC, log_result};

pub(crate) fn src(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "build ", config.build.src);

    let output = Command::new(KOTLINC)
        .current_dir(&config.project.dir)
        .arg(&config.build.src)
        .args(["-d", &config.build.output_src()])
        .output()
        .with_context(|| PartialConclusion::FAILED)?;

    log_result(output)
}

pub(crate) fn test(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "build ", config.build.test);

    let classpath = [
        &config.build.output_src(),
        "../kotlinc/lib/kotlin-test-junit5.jar",
        "../kotlinc/lib/kotlin-test.jar",
        "libs/junit-platform-console-standalone-1.9.2.jar",
    ].join(":");

    let output = Command::new(KOTLINC)
        .current_dir(&config.project.dir)
        .arg(&config.build.test)
        .args(["-cp", &classpath])
        .args(["-d", &config.build.output_test()])
        .output()
        .with_context(|| "failed  to build tests")?;

    log_result(output)
}
