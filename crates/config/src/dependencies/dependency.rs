use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;

use anyhow::bail;
use regex::Regex;

use crate::buildk;
use crate::dependencies::kind::Kind;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub kind: Kind,
    pub path: PathBuf,
    pub filename: PathBuf,
    pub url: String,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            Kind::Source => writeln!(f, "{:<26}{}:{}", "dependency", self.name, self.version),
            Kind::Test => writeln!(f, "{:<26}{}:{}", "test-dependency", self.name, self.version)
        }
    }
}

pub trait DependenciesKind {
    fn for_test(self) -> Vec<Dependency>;
    fn for_src(self) -> Vec<Dependency>;
}

impl DependenciesKind for Vec<Dependency> {
    fn for_test(self) -> Vec<Dependency> {
        self.into_iter().filter(|dep| dep.kind == Kind::Test).collect()
    }

    fn for_src(self) -> Vec<Dependency> {
        self.into_iter().filter(|dep| dep.kind == Kind::Source).collect()
    }
}

impl Dependency {
    pub fn is_cached(&self) -> bool {
        self.path.is_file()
    }

    pub fn from_toml(kind: &Kind, name: &str, item: &toml_edit::Value) -> anyhow::Result<Dependency> {
        if let Some(version) = item.as_str() {
            let info = dependency_info(name, version).unwrap();
            Ok(Self {
                name: name.into(),
                version: version.into(),
                kind: kind.clone(),
                path: info.path,
                filename: info.filename,
                url: info.url,
            })
        } else {
            bail!("Unresolved dependency, kind: {:?}, name: {name}, version: {item}", kind)
        }
    }
}

/// [name] "org.apache.kafka.kafka-clients" [version] "3.4.0"
fn dependency_info(name: &str, version: &str) -> anyhow::Result<DependencyInfo> {
    let after_last_slash = Regex::new(r"([^/]+)$").unwrap();
    let dependency = name.replace('.', "/");

    // todo: place this elsewhere
    let home = buildk::home_dir().unwrap();
    let cache = home.join("cache");

    match after_last_slash.find(&dependency) {
        None => bail!("artifact not found for dependency"),
        Some(artifact_name) => {
            match dependency.split('/').map(PathBuf::from).reduce(|a, b| a.join(b)) {
                None => bail!("relative path for dependency not deduced"),
                Some(relative_path) => {
                    let jar = format!("{}-{version}.jar", artifact_name.as_str());

                    Ok(DependencyInfo {
                        url: format!("https://repo1.maven.org/maven2/{dependency}/{version}/{jar}"),
                        // url: format!("https://repo.maven.apache.org/maven2/{dependency}/{version}/{jar}"),
                        path: cache.join(relative_path).join(&jar),
                        filename: PathBuf::from(jar),
                    })
                }
            }
        }
    }
}

struct DependencyInfo {
    pub path: PathBuf,
    pub filename: PathBuf,
    pub url: String,
}