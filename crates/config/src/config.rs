use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::buildk;
use crate::manifest::Manifest;

pub struct Config {
    /// buildk home dir `$HOME/.buildk`
    pub home: PathBuf,

    /// project manifest `buildk.toml`
    pub manifest: Manifest,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            manifest: Manifest::default(),
            home: match buildk::home_dir() {
                Ok(home_dir) => home_dir,
                Err(e) => panic!("Failed to construct config.home: {}", e)
            }
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "buildk.home:", self.home.display())?;
        write!(f, "{}", self.manifest)
    }
}
