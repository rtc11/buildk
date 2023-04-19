use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::{KOTLINC, PROJECT};

pub(crate) fn app() -> Result<String> {
    let output = Command::new(KOTLINC)
        .current_dir(PROJECT)
        .arg("src")
        .args(["-include-runtime", "-d", "build/app.jar"])
        // .arg("@app.options")
        // .arg("@classes")
        .output()
        .with_context(|| "release of app.jar failed")?;

    let stderr = String::from_utf8(output.stderr).with_context(||"failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(||"failed to read stdout");

    if !stderr.is_empty() {
        bail!(stderr)
    } else {
        stdout
    }
}

pub(crate) fn lib() -> Result<String> {
    let output = Command::new(KOTLINC)
        .current_dir(PROJECT)
        .arg("src")
        .args(["-d", "build/lib.jar"])
        // .arg("@lib.options")
        // .arg("@classes")
        .output()
        .with_context(|| "release of lib.jar failed")?;

    let stderr = String::from_utf8(output.stderr).with_context(||"failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(||"failed to read stdout");

    if !stderr.is_empty() {
        bail!(stderr)
    } else {
        stdout
    }
}
