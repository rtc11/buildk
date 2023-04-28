use std::fmt::{Display, Formatter};

use serde_derive::Deserialize;

#[derive(Deserialize, Clone, PartialEq, Debug, Hash)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub test: bool
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.test {
            writeln!(f, "{:<26}{}:{}", "test.dependency", self.name, self.version)
        } else {
            writeln!(f, "{:<26}{}:{}", "dependency", self.name, self.version)
        }
    }
}
