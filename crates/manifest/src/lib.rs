extern crate core;

use std::path::Path;
use std::str::FromStr;

use anyhow::{Context, Result};

pub mod repositories;
pub mod manifest;
pub mod config;
pub mod project;
pub mod dependencies;
pub mod modules;
pub mod home;

pub fn read_file(file: &Path) -> Result<String> {
    std::fs::read_to_string(file).context(format!("File not found: {}", file.display()))
}

pub(crate) enum Section {
    Project,
    Repositories,
    Module,
    Dependencies,
    TestDependencies,
    Kotlin,
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "project" => Section::Project,
            "repositories" => Section::Repositories,
            "module" => Section::Module,
            "dependencies" => Section::Dependencies,
            "test-dependencies" => Section::TestDependencies,
            "kotlin" => Section::Kotlin,
            _ => anyhow::bail!("Invalid section: {}", s),
        })
    }
}

