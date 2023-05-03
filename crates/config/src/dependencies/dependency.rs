use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

use crate::dependencies::kind::Kind;

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
        match self.kind {
            Kind::Production => writeln!(f, "{:<26}{}:{}", "dependency", self.name, self.version),
            Kind::Test => writeln!(f, "{:<26}{}:{}", "test-dependency", self.name, self.version)
        }
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

    pub fn from_toml(kind: &Kind, name: &str, item: &toml_edit::Value) -> anyhow::Result<Dependency> {
        if let Some(version) = item.as_str() {
            let dependency = Self::new(kind.clone(), name, version);
            Ok(dependency)
        } else {
            anyhow::bail!("Unresolved dependency, kind: {:?}, name: {name}, version: {item}", kind)
        }
    }
}

