use std::process::Command;

use anyhow::{Context, Result};
use fs::toml::Config;

use crate::{KOTLINC, log_result};

pub(crate) fn jar(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "target ", config.output_jar());

    let runtime = if config.is_app() { "-include-runtime" } else { "" };

    let output = Command::new(KOTLINC)
        .current_dir(&config.project.dir)
        .arg(&config.src_dir())
        .arg(runtime)
        .args(["-d", &config.output_jar()])
        .output()
        .with_context(|| "release of jar failed")?;

    log_result(output)
}
