use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Clone)]
pub struct Home {
    path: PathBuf,
}

impl Default for Home {
    fn default() -> Self {
        Self {
            path: home::home_dir()
                .map(|it| it.join(".buildk"))
                .expect("buildk could not find its home directory $HOME/.buildk"),
        }
    }
}

pub fn cache_location() -> PathBuf {
    home::home_dir()
        .expect("home directory")
        .join(".buildk")
        .join("cache")
}

impl Display for Home {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "home:", self.path.display())
    }
}
