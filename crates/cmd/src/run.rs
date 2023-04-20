use std::process::Command;

use anyhow::{bail, Context, Result};
use console::PartialConclusion;
use fs::toml::Config;

pub(crate) fn app(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "run", config.main());

    let output = Command::new("java")
        .current_dir(config.project_dir())
        .arg("-cp")
        .arg(config.build_src_dir())
        .arg(config.main())
        .output()
        .with_context(|| "failed to run app")?;

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
