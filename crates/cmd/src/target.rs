use std::process::Command;

use anyhow::{Context, Result};
use console::PartialConclusion;
use fs::toml::Config;

use crate::{KOTLINC, log_result};

pub(crate) fn jar(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "target ", config.build.target());

    let runtime = if config.is_app() { "-include-runtime" } else { "" };

    let output = Command::new(KOTLINC)
        .current_dir(&config.project.dir)
        .arg(&config.build.src)
        .arg(runtime)
        .args(["-d", &config.build.target()])
        .output()
        .with_context(|| PartialConclusion::FAILED)?;

    log_result(output)
}
