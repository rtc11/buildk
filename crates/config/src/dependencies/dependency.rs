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
    pub target_dir: PathBuf,
    pub url: String,
    pub jar: String,
    pub sources: String,
    pub pom: String,
    pub module: String,
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
    pub fn jar_path(&self) -> PathBuf { self.target_dir.join(&self.jar)}
    pub fn is_cached(&self) -> bool {
        self.target_dir.join(&self.jar).is_file()
    }

    pub fn from_toml(kind: &Kind, name: &str, item: &toml_edit::Value) -> anyhow::Result<Dependency> {
        if let Some(version) = item.as_str() {
            let info = dependency_info(name, version).unwrap();
            Ok(Self {
                name: name.into(),
                version: version.into(),
                kind: kind.clone(),
                target_dir: info.target_dir,
                url: info.url,
                jar: format!("{}.jar", info.file_suffix),
                sources: format!("{}-sources.jar", info.file_suffix),
                pom: format!("{}.pom", info.file_suffix),
                module: format!("{}.module", info.file_suffix)
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
                    Ok(DependencyInfo {
                        url: format!("https://repo1.maven.org/maven2/{dependency}/{version}/"),
                        file_suffix: format!("{}-{version}", artifact_name.as_str()),
                        // jar_file: format!("{file_suffix}.jar"),
                        // sources_file: format!("{file_suffix}-sources.jar"),
                        // module_file: format!("{file_suffix}.module"),
                        // pom_file: format!("{file_suffix}.pom"),
                        target_dir: cache.join(relative_path),
                        // path: cache.join(relative_path).join(&jar_file),
                        // filename: PathBuf::from(jar),
                        // name: format!("{}-{version}", artifact_name.as_str()),
                    })
                }
            }
        }
    }
}

struct DependencyInfo {
    pub url: String,
    pub file_suffix: String,
    // pub jar_file: String,
    // pub sources_file: String,
    // pub module_file: String,
    // pub pom_file: String,
    pub target_dir: PathBuf,
    // pub name: String,
    // pub path: PathBuf,
    // pub filename: PathBuf,
    // pub jar_url: String,
    // pub sources_url: String,
    // pub module_url: String,
    // pub pom_url: String,
}
