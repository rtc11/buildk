use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::{KOTLINC, PROJECT};

pub(crate) fn src() -> Result<String> {
    let output = Command::new(KOTLINC)
        .current_dir(PROJECT)
        .arg("src")
        .args(["-d", "build/src"])
        .output()
        .with_context(|| "build failed")?;

    let stderr = String::from_utf8(output.stderr).with_context(|| "failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(|| "failed to read stdout");

    if !stderr.is_empty() {
        bail!(stderr)
    } else {
        stdout
    }
}

pub(crate) fn test() -> Result<String> {
    let classpath = [
        "build/src",
        "../kotlinc/lib/kotlin-test-junit5.jar",
        "../kotlinc/lib/kotlin-test.jar",
        "libs/junit-platform-console-standalone-1.9.2.jar",
    ];

    let output = Command::new(KOTLINC)
        .current_dir(PROJECT)
        .arg("test")
        .args(["-cp", &classpath.join(":")])
        .args(["-d", "build/test"])
        .output()
        .with_context(|| "failed  to build tests")?;

    let stderr = String::from_utf8(output.stderr).with_context(||"failed to read stderr")?;
    let stdout = String::from_utf8(output.stdout).with_context(||"failed to read stdout");

    if !stderr.is_empty() {
        bail!(stderr)
    } else {
        stdout
    }
}
