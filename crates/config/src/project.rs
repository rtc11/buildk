use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Project {
    main: String,
    #[serde(default = "Project::default_dir")]
    pub dir: PathBuf,
}

impl Project {
    pub fn main_class(&self) -> String { self.main.replace(".kt", "Kt") }

    fn default_dir() -> PathBuf {
        env::current_dir().expect("could not find the current directory")
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project.main:", self.main)?;
        writeln!(f, "{:<26}{}", "project.dir:", self.dir.display())
    }
}
