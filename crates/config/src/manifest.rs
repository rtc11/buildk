use std::fmt::{Display, Formatter};
use serde_derive::Deserialize;
use crate::build::Build;
use crate::kotlin::Kotlin;
use crate::project::Project;

#[derive(Deserialize, Clone)]
pub struct Manifest {
    pub project: Project,
    pub build: Build,
    pub kotlin: Kotlin,
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        write!(f, "{}", self.build)?;
        write!(f, "{}", self.kotlin)
    }
}
