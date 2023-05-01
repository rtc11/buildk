use std::fmt::{Display, Formatter};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    PRODUCTION,
    TEST,
}

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub kind: Kind,
    pub path: Option<PathBuf>,
    pub url: Option<PathBuf>,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}:{}", "dependency", self.name, self.version)
        // writeln!(f, "{:<26}{}", "dependency.version", self.version)?;
        // writeln!(f, "{:<26}{:?}", "dependency.kind", self.kind)?;
        // writeln!(f, "{:<26}{}", "dependency.path", self.path.unwrap_or_default().display())?;
        // writeln!(f, "{:<26}{}", "dependency.url", self.url.unwrap_or_default().display())
    }
}

impl Dependency {
    pub fn new(kind: Kind, name: &str, version: &str) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            kind,
            path: None,
            url: None,
        }
    }

    pub fn from_toml(kind: Kind, name: &str, item: &toml_edit::Item) -> anyhow::Result<Dependency> {
        if let Some(version) = item.as_str() {
            let dependency = Self::new(kind, name, version);
            Ok(dependency)
        } else {
            anyhow::bail!("Unresolved dependency, kind: {:?}, name: {name}, version: {item}", kind)
        }
    }
}