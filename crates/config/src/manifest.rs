use std::fmt::{Display, Formatter};
use std::path::Path;

use util::get_kotlin_home;

use crate::dependencies::dependencies;
use crate::dependencies::dependency::Dependency;
use crate::modules::module::Module;
use crate::project::project;
use crate::project::project::Project;
use crate::read_file;

pub struct Manifest {
    pub project: Project,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
}

impl Default for Manifest {
    fn default() -> Self {
        let content = read_file(manifest_path()).expect("buildk.toml not found.");
        let toml = content.parse().expect("Manifest not valid TOML.");
        Manifest {
            project: project(&toml).unwrap_or_default(),
            modules: vec![],
            dependencies: dependencies(&toml),
        }
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
fn manifest_path() -> &'static Path { Path::new("example/buildk.toml") }

#[cfg(not(debug_assertions))]
fn manifest_path() -> &'static Path { Path::new("buildk.toml") }
