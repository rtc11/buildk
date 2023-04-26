use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::manifest::Manifest;
use crate::{buildk_home_dir, read_file};

pub struct Config {
    /// current working directory
    pub cwd: PathBuf,
    /// buildk home dir `$HOME/.buildk`
    pub home: PathBuf,
    /// project manifest `buildk.toml`
    pub manifest: Manifest,
}

impl Default for Config {
    fn default() -> Self {
        match Config::new() {
            Ok(config) => config,
            Err(e) => panic!("Failed to configure: {e}")
        }
    }
}

impl Config {
    fn new() -> anyhow::Result<Config> {
        let manifest = read_file::<Manifest>(Path::new(Self::manifest_path()))?;
        let cwd = manifest.project.dir.clone();
        let home = buildk_home_dir().with_context(|| "buildk could not find its home directory")?;
        Ok(Config { cwd, home, manifest })
    }

    #[cfg(debug_assertions)]
    fn manifest_path() -> &'static str { "test/buildk.toml" }

    #[cfg(not(debug_assertions))]
    fn manifest_path() -> &'static str { "buildk.toml" }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "cwd:", self.cwd.display())?;
        writeln!(f, "{:<26}{}", "buildk.home:", self.home.display())?;
        write!(f, "{}", self.manifest)
    }
}
