use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Kotlin {
    /// path to kotlin bin directory. TODO: require envar KOTLIN_HOME?
    #[serde(default = "default_kotlin_path")]
    pub path: PathBuf,
}

fn default_kotlin_path() -> PathBuf { PathBuf::from("..kotlinc") }

impl Kotlin {
    pub fn bin(&self) -> PathBuf { self.path.join("bin") }
}

impl Display for Kotlin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "kotlin.path:", self.path.display())
    }
}
