use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::PROJECT;

pub(crate) fn run() -> Result<String> {
    let output = Command::new("java")
        .current_dir(PROJECT)
        .args(["-jar", "libs/junit-platform-console-standalone-1.9.2.jar"])
        .args(["-cp", "build/test:build/src"])
        // .args(["--select-package", "no.tordly.test"])
        .args(["--select-class", "PrefixTest"])
        .output()
        .with_context(|| "failed to run tests")?;

    let stderr = String::from_utf8(output.stderr).with_context(||"failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(||"failed to read stdout");

    if !stderr.is_empty() {
        bail!(stderr)
    } else {
        stdout
    }
}
