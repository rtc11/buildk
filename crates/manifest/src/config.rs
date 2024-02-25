use std::fmt::{Display, Formatter};

use crate::home::Home;
use crate::manifest::Manifest;

#[derive(Clone)]
pub struct Config {
    /// $HOME/.buildk
    pub home: Home,

    /// `buildk.toml`
    pub manifest: Option<Manifest>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Config {
            home: Home::new(),
            manifest: Manifest::try_new().ok(),
        }
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.home)?;

        if let Some(manifest) = &self.manifest {
            write!(f, "{}", manifest)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn no_manifest() {
        let config = Config::new();
        assert!(config.manifest.is_none());
    }
}
