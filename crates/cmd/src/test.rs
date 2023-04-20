use std::process::Command;

use anyhow::{bail, Context, Result};
use console::PartialConclusion;
use fs::toml::Config;

pub(crate) fn run(config: &Config) -> Result<String> {
    print!("▸ {:<7}", "run test");

    let classpath = [
        config.build_dir(),
        config.test_dir()
    ].join(":");

    let output = Command::new("java")
        .current_dir(config.project_dir())
        .args(["-jar", "libs/junit-platform-console-standalone-1.9.2.jar"])
        .args(["-cp", &classpath])
        // .args(["--select-package", "no.tordly.test"])
        .args(["--select-class", "PrefixTest"])
        .output()
        .with_context(|| "failed to run tests")?;

    let stderr = String::from_utf8(output.stderr).with_context(||"failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(||"failed to read stdout");

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
