use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Home {
    path: PathBuf,
}

impl Default for Home {
    fn default() -> Self {
        Self::new()
    }
}

impl Home {
    pub fn new() -> Self {
        Self {
            path: home::home_dir()
                .map(|it| it.join(".buildk"))
                .expect("buildk could not find its home directory $HOME/.buildk")
        }
    }
}

impl Display for Home {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "home:", self.path.display())
    }
}
