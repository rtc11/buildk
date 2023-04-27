use std::fmt::{Display, Formatter};
use serde_derive::Deserialize;
use crate::build::Build;
use crate::project::Project;

#[derive(Deserialize, Clone)]
pub struct Manifest {
    pub project: Project,
    pub build: Build,
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        write!(f, "{}", self.build)?;
        writeln!(f, "{:<26}{:?}", "kotlin.path:", option_env!("KOTLIN_HOME"))
    }
}
