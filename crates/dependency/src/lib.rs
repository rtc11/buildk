use std::collections::BTreeSet;
use std::fmt::Display;
use std::path::{Path, PathBuf};

pub mod parser;

pub trait Parser<T>
where
    T: Ord,
{
    fn parse(path: PathBuf) -> BTreeSet<T>;
}

pub fn resolve_descriptor(path: &Path) -> Option<PathBuf> {
    let gradle_descriptor = path.join("maven.xml");
    if gradle_descriptor.exists() {
        return Some(gradle_descriptor);
    }

    let maven_descriptor = path.join("gradle.json");
    if maven_descriptor.exists() {
        return Some(maven_descriptor);
    }

    None
}

pub fn cache_location() -> PathBuf {
    home::home_dir()
        .expect("home directory")
        .join(".buildk")
        .join("cache")
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Clone)]
pub struct Package {
    pub name: String,
    pub namespace: Option<String>,
    pub version: String,
    pub kind: PackageKind,
    pub location: PathBuf,
}

impl Package {
    pub fn new(
        name: String,
        namespace: Option<String>,
        version: String,
        kind: PackageKind,
    ) -> Self {
        let location = match &namespace {
            Some(ns) => cache_location().join(&ns).join(&name).join(&version),
            None => cache_location().join(&name).join(&version),
        };

        Package {
            name,
            namespace,
            version,
            kind,
            location,
        }
    }
    // todo: if one transitive dep has previously failed, this is not good enough for a check
    pub fn is_cached(&self) -> bool {
        let jar = self.jar_absolute_path();
        let descriptor = resolve_descriptor(&self.location);

        if !jar.exists() {
            return false;
        } else if jar.metadata().unwrap().len() == 0 {
            println!("Jar found {}, but was empty", self.name);
            return false;
        }

        if descriptor.is_none() {
            println!("No descriptor found for {}", self.name);
            return false;
        } else if descriptor.unwrap().metadata().unwrap().len() == 0 {
            println!("Desciptor found {}, but was empty", self.name);
            return false;
        }

        true
    }

    pub fn transitives(&self) -> Vec<Package> {
        parser::parse(&self.location).into_iter().collect()
    }

    pub fn jar_absolute_path(&self) -> PathBuf {
        self.location.join("pkg.jar")
    }

    pub fn classpath(&self) -> String {
        self.transitives()
            .clone()
            .into_iter()
            .map(|pkg| pkg.jar_absolute_path())
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .as_slice()
            .join(":")
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Clone, Debug)]
pub enum PackageKind {
    #[default]
    Compile,
    Runtime,
    Test,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct MavenPackage {
    group: String,
    artifact: String,
    version: String,
    scope: MavenScope,
    location: Option<PathBuf>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub enum MavenScope {
    #[default]
    Compile, // default, available at compile-time and runtime
    Provided, // available at compile-time only, but is still required at runtime
    Runtime,  // only available at runtime
    Test,     // only available at test-compile-time and test-runtime
    System,   // required at compile-time and runtime, but is not included in the project
}
