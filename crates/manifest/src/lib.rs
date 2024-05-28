use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result};
use packages::Packages;
use project::Project;
use repos::Repos;

pub mod config;
pub mod home;
pub mod packages;
pub mod project;
pub mod repos;

pub fn read_file(file: &Path) -> Result<String> {
    std::fs::read_to_string(file).context(format!("File not found: {}", file.display()))
}

pub(crate) enum Section {
    Project,
    Repos,
    CompileDeps,
    RuntimeDeps,
    TestDeps,
    Kotlin,
    Java,
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "project" => Section::Project,
            "repos" => Section::Repos,
            "compile" => Section::CompileDeps,
            "runtime" => Section::RuntimeDeps,
            "test" => Section::TestDeps,
            "kotlin" => Section::Kotlin,
            "java" => Section::Java,
            _ => anyhow::bail!("Invalid section: {}", s),
        })
    }
}

#[derive(Clone)]
pub struct Manifest {
    pub project: Project,
    pub repos: Repos,
    pub compile_deps: Packages,
    pub runtime_deps: Packages,
    pub test_deps: Packages,
    pub kotlin_home: Option<PathBuf>,
    pub java_home: Option<PathBuf>,
    pub all_packages: Packages, // TODO: can we remove this?
}

impl Manifest {
    pub fn try_new() -> anyhow::Result<Manifest> {
        let path = Path::new("buildk.toml");
        let content = read_file(path).context("buildk.toml not found.")?;

        let toml = match content.parse().context("Manifest not valid TOML.") {
            Ok(toml) => toml,
            Err(err) => {
                eprintln!("Failed to parse TOML: {}", err);
                anyhow::bail!("Failed to parse TOML: {}", err)
            }
        };

        let packages = Packages::from(&toml);

        Ok(Manifest {
            project: Project::from(toml.clone()),
            repos: Repos::from(toml.clone()),
            compile_deps: Packages::new(packages.compile()),
            runtime_deps: Packages::new(packages.runtime()),
            test_deps: Packages::new(packages.test()),
            kotlin_home: kotlin_home(&toml),
            java_home: java_home(&toml),
            all_packages: packages,
        })
    }
}

fn kotlin_home(manifest: &toml_edit::DocumentMut) -> Option<PathBuf> {
    let kotlins = manifest
        .as_table()
        .into_iter()
        .flat_map(|(key, value)| match Section::from_str(key) {
            Ok(Section::Kotlin) => match value.as_table() {
                None => vec![],
                Some(table) => table
                    .iter()
                    .map(|(_, path)| PathBuf::from(path.as_str().unwrap()))
                    .collect(),
            },
            _ => vec![],
        })
        .collect::<Vec<_>>();
        
        kotlins.first().cloned()
}

fn java_home(manifest: &toml_edit::DocumentMut) -> Option<PathBuf> {
    let java = manifest
        .as_table()
        .into_iter()
        .flat_map(|(key, value)| match Section::from_str(key) {
            Ok(Section::Java) => match value.as_table() {
                None => vec![],
                Some(table) => table
                    .iter()
                    .map(|(_, path)| PathBuf::from(path.as_str().unwrap()))
                    .collect(),
            },
            _ => vec![],
        })
        .collect::<Vec<_>>();
        
        java.first().cloned()
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;

        if self.kotlin_home.is_some() {
            write!(f, "{}", self.kotlin_home.clone().unwrap().display())?;
        }

        for repo in self.repos.repos.iter() {
            write!(f, "{}", repo)?;
        }

        writeln!(f, "{:<26}{}", "Compile", self.compile_deps)?;
        writeln!(f, "{:<26}{}", "Runtime", self.runtime_deps)?;
        writeln!(f, "{:<26}{}", "Test", self.test_deps)?;
        write!(f, "")
    }
}
