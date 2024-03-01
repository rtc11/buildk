use std::fmt::Display;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::dependencies::{dependencies, DependenciesKind, Dependency};
use crate::modules::Module;
use crate::project::project;
use crate::project::Project;
use crate::read_file;
use crate::repositories::{repositories, Repository};

#[derive(Clone)]
pub struct Manifest {
    pub project: Project,
    pub repositories: Vec<Repository>,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
    pub kotlin_home: Option<PathBuf>,
}

impl Manifest {
    pub fn try_new() -> anyhow::Result<Manifest> {
        let path = Path::new("buildk.toml");
        let content = read_file(path).context("buildk.toml not found.")?;
        let toml = content.parse().context("Manifest not valid TOML.")?;

        Ok(Manifest {
            project: project(&toml).unwrap_or_default(),
            repositories: repositories(&toml),
            modules: vec![],
            dependencies: dependencies(&toml),
            kotlin_home: kotlin_home(&toml),
        })
    }
}

fn kotlin_home(manifest: &toml_edit::Document) -> Option<PathBuf> {
    manifest.as_table()
        .get("kotlin")
        .and_then(|it| it.as_str())
        .map(PathBuf::from)
}


impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;

        if self.kotlin_home.is_some() {
            write!(f, "{}", self.kotlin_home.clone().unwrap().display())?;
        }

        for repo in self.repositories.iter() {
            write!(f, "{}", repo)?;
        }

        let plat_deps = self.dependencies.for_platform().iter()
            .map(|dep| format!("{dep}"))
            .collect::<Vec<_>>()
            .join(":");
        writeln!(f, "{:<26}{}", "Platform dependencies", plat_deps)?;

        let src_deps = self.dependencies.for_src().iter()
            .map(|dep| format!("{dep}"))
            .collect::<Vec<_>>()
            .join(":");
        writeln!(f, "{:<26}{}", "Source dependencies", src_deps)?;

        let test_deps = self.dependencies.for_test().iter()
            .map(|dep| format!("{dep}"))
            .collect::<Vec<_>>()
            .join(":");

        writeln!(f, "{:<26}{}", "Test dependencies", test_deps)?;

        for module in self.modules.iter() {
            write!(f, "{}", module)?;
        }

        write!(f, "")
    }
}

