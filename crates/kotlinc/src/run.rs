use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::PROJECT;

pub(crate) fn app(main: &str) -> Result<String> {
    let output = Command::new("java")
        .current_dir(PROJECT)
        .arg("-cp")
        .arg("build/src")
        .arg(main)
        .output()
        .with_context(|| "failed to run app")?;

    let stderr = String::from_utf8(output.stderr).with_context(||"failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(||"failed to read stdout");

    if !stderr.is_empty() {
        bail!(stderr)
    } else {
        stdout
    }
}
