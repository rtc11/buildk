use std::process::Command;

use anyhow::{bail, Context, Result};
use console::PartialConclusion;
use fs::toml::Config;

use crate::KOTLINC;

pub(crate) fn jar(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "target ", config.target());

    let runtime = if config.is_app() { "-include-runtime" } else { "" };

    let output = Command::new(KOTLINC)
        .current_dir(config.project_dir())
        .arg(config.src_dir())
        .arg(runtime)
        .args(["-d", &config.target()])
        .output()
        .with_context(|| "release of jar failed")?;

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
