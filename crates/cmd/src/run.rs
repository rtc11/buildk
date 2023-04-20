use std::process::Command;

use anyhow::{Context, Result};
use fs::toml::Config;
use crate::log_result;

pub(crate) fn app(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "run", config.project.main());

    let output = Command::new("java")
        .current_dir(&config.project.dir)
        .arg("-cp")
        .arg(config.output_src())
        .arg(config.project.main())
        .output()
        .with_context(|| "failed to run app")?;

    log_result(output)
}
