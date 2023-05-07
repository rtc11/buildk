use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::buildk;
use crate::classpath::Classpath;
use crate::manifest::Manifest;

pub struct Config {
    /// `$HOME/.buildk`
    pub home: PathBuf,
    /// `buildk.toml`
    pub manifest: Manifest,
    pub classpath: Classpath,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            manifest: Manifest::default(),
            home: match buildk::home_dir() {
                Ok(home_dir) => home_dir,
                Err(e) => panic!("Failed to construct config.home: {}", e)
            },
            classpath: Classpath::default()
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "buildk.home:", self.home.display())?;
        write!(f, "{}", self.manifest)
    }
}
