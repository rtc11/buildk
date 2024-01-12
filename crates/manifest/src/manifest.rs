use std::fmt::{Display, Formatter};
use std::path::Path;

use util::get_kotlin_home;

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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        writeln!(f, "{:<26}{}", "kotlin.path", get_kotlin_home().display())?;
        self.dependencies.iter().try_for_each(|dependency| write!(f, "{}", dependency))
    }
}


