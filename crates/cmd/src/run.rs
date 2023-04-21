use std::process::Command;

use anyhow::{Context, Result};
use console::PartialConclusion;
use fs::toml::Config;
use crate::log_result;

pub(crate) fn app(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "run", config.project.main_class());

    let output = Command::new("java")
        .current_dir(&config.project.dir)
        .arg("-cp")
        .arg(&config.build.src)
        .arg(config.project.main_class())
        .output()
        .with_context(|| PartialConclusion::FAILED)?;

    log_result(output)
}
