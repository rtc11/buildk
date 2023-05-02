use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

use anyhow::Context;

use util::get_kotlin_home;

use crate::dependencies::{dependencies, Dependency};
use crate::module::Module;
use crate::project::{Project, project};
use crate::read_file;

pub struct Manifest {
    pub project: Project,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
}

impl Default for Manifest {
    fn default() -> Self {
        let content = read_file(manifest_path()).unwrap();
        let toml = TomlParser::from_str(&content).unwrap();

        Manifest {
            project: toml.project().unwrap_or_default(),
            modules: toml.modules(),
            dependencies: toml.dependencies(),
        }
    }
}

pub struct TomlParser {
    data: toml_edit::Document,
}

pub enum Section {
    Project,
    Module,
    Dependencies,
    TestDependencies,
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "project" => Section::Project,
            "module" => Section::Module,
            "dependencies" => Section::Dependencies,
            "test-dependencies" => Section::TestDependencies,
            _ => anyhow::bail!("Invalid section: {}", s),
        })
    }
}

impl TomlParser {
    pub fn project(&self) -> Option<Project> {
        project(&self.data)
    }

    pub fn modules(&self) -> Vec<Module> {
        vec![]
    }

    pub fn dependencies(&self) -> Vec<Dependency> {
        dependencies(&self.data)
    }
}

impl FromStr for TomlParser {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let manifest: toml_edit::Document = s.parse().context("Manifest not valid TOML.")?;
        Ok(TomlParser { data: manifest })
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        writeln!(f, "{:<26}{}", "kotlin.path", get_kotlin_home().display())?;
        self.dependencies.iter().try_for_each(|dependency| write!(f, "{}", dependency))
    }
}

#[cfg(debug_assertions)]
fn manifest_path() -> &'static Path { Path::new("test/buildk.toml") }

#[cfg(not(debug_assertions))]
fn manifest_path() -> &'static Path { Path::new("buildk.toml") }
