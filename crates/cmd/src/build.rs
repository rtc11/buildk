use std::process::Command;

use anyhow::{bail, Context, Result};

use console::PartialConclusion;
use fs::toml::Config;

use crate::KOTLINC;

pub(crate) fn src(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}/", "build ", config.src_dir());

    let output = Command::new(KOTLINC)
        .current_dir(config.project_dir())
        .arg(config.src_dir())
        .args(["-d", &config.build_src_dir()])
        .output()
        .with_context(|| "build failed")?;

    let stderr = String::from_utf8(output.stderr).with_context(|| "failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(|| "failed to read stdout");

    if !stderr.is_empty() {
        println!("{}", PartialConclusion::FAILED);
        bail!(stderr)
    } else {
        match &stdout {
            Ok(_) => println!("{}", PartialConclusion::SUCCESS),
            Err(_) => println!("{}", PartialConclusion::FAILED),
        }
        stdout
    }
}

pub(crate) fn test(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}/", "build ", config.test_dir());

    let classpath = [
        &config.build_src_dir(),
        "../kotlinc/lib/kotlin-test-junit5.jar",
        "../kotlinc/lib/kotlin-test.jar",
        "libs/junit-platform-console-standalone-1.9.2.jar",
    ];

    let output = Command::new(KOTLINC)
        .current_dir(config.project_dir())
        .arg(config.test_dir())
        .args(["-cp", &classpath.join(":")])
        .args(["-d", &config.build_test_dir()])
        .output()
        .with_context(|| "failed  to build tests")?;

    let stderr = String::from_utf8(output.stderr).with_context(|| "failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(|| "failed to read stdout");

    if !stderr.is_empty() {
        println!("{}", PartialConclusion::FAILED);
        bail!(stderr)
    } else {
        match &stdout {
            Ok(_) => println!("{}", PartialConclusion::SUCCESS),
            Err(_) => println!("{}", PartialConclusion::FAILED),
        }
        stdout
    }
}
