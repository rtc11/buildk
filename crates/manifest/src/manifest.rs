use std::path::Path;

use util::get_kotlin_home;
use util::terminal::Printable;

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

impl Printable for Manifest {
    fn print(&self, terminal: &mut util::terminal::Terminal) {
        self.project.print(terminal);
        terminal.print(&format!("{:<26}{}", "kotlin.path", get_kotlin_home().display()));
        self.dependencies.iter().for_each(|dependency| dependency.print(terminal));
    }
}

