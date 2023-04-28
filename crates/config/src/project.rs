use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Project {
    #[serde(default = "Project::default_main")]
    main: String,
    #[serde(default = "Project::default_path")]
    pub path: PathBuf,
}

impl Default for Project {
    fn default() -> Self {
        Project {
            main: Project::default_main(),
            path: Project::default_path(),
        }
    }
}

impl Project {
    pub fn main_class(&self) -> String {
        self.main.replace(".kt", "Kt")
    }

    fn default_path() -> PathBuf {
        env::current_dir().expect("could not find the current directory")
    }

    fn default_main() -> String {
        "Main.kt".to_string()
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project.main", self.main)?;
        writeln!(f, "{:<26}{}", "project.path", self.path.display())
    }
}
