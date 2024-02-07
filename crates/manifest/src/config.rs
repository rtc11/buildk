use std::path::PathBuf;

use util::terminal::{Terminal, Printable};

use crate::classpath::Classpath;
use crate::manifest::Manifest;

#[derive(Default, Clone)]
pub struct Config {
    /// $HOME/.buildk
    pub home: Buildk,

    /// `buildk.toml`
    pub manifest: Manifest,


    pub _classpath: Classpath,
}

#[derive(Clone)]
pub struct Buildk {
    home: PathBuf
}

impl Default for Buildk {
    fn default() -> Self {
        let home = home::home_dir()
            .map(|it| it.join(".buildk"))
            .expect("buildk could not find its home directory");

        Self {
            home
        }
    }
}

impl Printable for Config {
    fn print(&self, terminal: &mut Terminal) {
        self.home.print(terminal);
        self.manifest.print(terminal);
    }
}

impl Printable for Buildk {
    fn print(&self, terminal: &mut Terminal) {
        terminal.print(&format!("{:<26}{}", "buildk.home:", self.home.display()));
    }
}

