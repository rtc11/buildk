use std::fmt::Display;
use std::path::Path;

use crate::dependencies::{dependencies, Dependency};
use crate::modules::Module;
use crate::project::project;
use crate::project::Project;
use crate::read_file;
use crate::repositories::{Repository, repositories};

#[derive(Clone)]
pub struct Manifest {
    pub project: Project,
    pub repositories: Vec<Repository>,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
}

impl Default for Manifest {
    fn default() -> Self {
        let path = Path::new("buildk.toml");
        let content = read_file(path).expect("buildk.toml not found.");
        let toml = content.parse().expect("Manifest not valid TOML.");

        Manifest {
            project: project(&toml).unwrap_or_default(),
            repositories: repositories(&toml),
            modules: vec![],
            dependencies: dependencies(&toml),
        }
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;

        for repo in self.repositories.iter() {
            write!(f, "{}", repo)?;
        }

        for dep in self.dependencies.iter() {
            write!(f, "{}", dep)?;
        }

        for module in self.modules.iter() {
            write!(f, "{}", module)?;
        }

        write!(f, "")
        
    }
}

