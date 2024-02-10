use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use util::get_kotlin_home;

use crate::classpath::Classpath;
use crate::manifest::Manifest;

#[derive(Default, Clone)]
pub struct Config {
    /// $HOME/.buildk
    pub home: Home,

    /// `buildk.toml`
    pub manifest: Manifest,


    pub _classpath: Classpath,
}

#[derive(Clone)]
pub struct Home {
    path: PathBuf
}

impl Default for Home {
    fn default() -> Self {
        Self {
            path: home::home_dir()
                .map(|it| it.join(".buildk"))
                .expect("buildk could not find its home directory")
        }
    }
}

impl Display for Home {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "home:", self.path.display())
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.home)?;
        write!(f, "{}", self.manifest)?;
        writeln!(f, "{:<26}{}", "kotlin.home", get_kotlin_home().display())
    }
}

