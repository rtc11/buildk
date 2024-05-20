use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use gradle_parser::GradleParser;
use util::DEBUG;

use crate::maven_parser::MavenParser;

mod buildk_parser;
pub mod gradle_parser;
mod ivy_parser;
pub mod maven_parser;

pub trait Parser<T>
where
    T: Ord,
{
    fn parse(path: PathBuf) -> BTreeSet<T>;
}

pub fn resolve(jar: &Path) -> BTreeSet<Dependency> {
    let mut descriptor = jar.to_path_buf();

    descriptor.set_extension("buildk");
    if descriptor.exists() {
        println!("buildk package descriptor exists, but parser is not implemented yet");
    }

    descriptor.set_extension("module");
    if descriptor.exists() {
        return GradleParser::parse(descriptor.to_path_buf());
    }

    descriptor.set_extension("pom");
    if descriptor.exists() {
        return MavenParser::parse(descriptor.to_path_buf());
    }

    descriptor.set_extension("ivy");
    if descriptor.exists() {
        if DEBUG {
            println!("ivy package descriptor exists, but parser is not implemented yet");
        }
    }

    if DEBUG {
        println!("package descriptor not found: {:?}", descriptor);
    }

    BTreeSet::default()
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub struct Dependency {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub kind: Kind,
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
pub enum Kind {
    Compile,
    Test,
}

impl Dependency {
    pub fn new(group: String, artifact: String, version: String, kind: Kind) -> Self {
        Self {
            group,
            artifact,
            version,
            kind,
        }
    }
}
