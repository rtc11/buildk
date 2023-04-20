use std::process::Command;

use anyhow::{Context, Result};

use fs::toml::Config;

use crate::{KOTLINC, log_result};

pub(crate) fn src(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "build ", config.src_dir());

    let output = Command::new(KOTLINC)
        .current_dir(&config.project.dir)
        .arg(config.src_dir())
        .args(["-d", &config.output_src()])
        .output()
        .with_context(|| "build failed")?;

    log_result(output)
}

pub(crate) fn test(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "build ", config.test_dir());

    let classpath = [
        &config.output_src(),
        "../kotlinc/lib/kotlin-test-junit5.jar",
        "../kotlinc/lib/kotlin-test.jar",
        "libs/junit-platform-console-standalone-1.9.2.jar",
    ].join(":");

    let output = Command::new(KOTLINC)
        .current_dir(&config.project.dir)
        .arg(config.test_dir())
        .args(["-cp", &classpath])
        .args(["-d", &config.output_test()])
        .output()
        .with_context(|| "failed  to build tests")?;

    log_result(output)
}
