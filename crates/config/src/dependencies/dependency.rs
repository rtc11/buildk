use std::fmt::{Debug, Display, Formatter};
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::bail;
use regex::Regex;
use xml::EventReader;
use xml::reader::XmlEvent;

use crate::buildk;
use crate::dependencies::kind::Kind;

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub kind: Kind,
    /// Directory
    pub target_dir: PathBuf,
    /// url to the artifact directory that contains all the files.
    pub url: String,
    /// Filename
    pub jar: String,
    /// Filename
    pub sources: String,
    /// Filename
    pub pom: String,
    /// Filename
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
    pub fn jar_path(&self) -> PathBuf { self.target_dir.join(&self.jar) }
    pub fn is_cached(&self) -> bool {
        self.target_dir.join(&self.jar).is_file()
    }

    pub fn new(kind: &Kind, name: &str, version: &str) -> Option<Dependency> {
        dependency_info(name, version).map(|info| {
            Self {
                name: name.into(),
                version: version.into(),
                kind: kind.clone(),
                target_dir: info.target_dir,
                url: info.url,
                jar: format!("{}.jar", info.file_suffix),
                sources: format!("{}-sources.jar", info.file_suffix),
                pom: format!("{}.pom", info.file_suffix),
                module: format!("{}.module", info.file_suffix),
            }
        }).ok()
    }

    pub fn from_toml(kind: &Kind, name: &str, item: &toml_edit::Value) -> Option<Dependency> {
        if let Some(version) = item.as_str() {
            Self::new(kind, name, version)
        } else {
            None
        }
        // if let Some(version) = item.as_str() {
        //     Ok(Self::new(kind, name, version))
        // } else {
        //     bail!("Unresolved dependency, kind: {:?}, name: {name}, version: {item}", kind)
        // }
    }

    pub fn transitives(&self) -> Vec<Dependency> {
        self.target_dir.join(&self.pom).to_dependencies(&self.kind)
    }
}

impl Pom for PathBuf {
    fn to_dependencies(&self, kind: &Kind) -> Vec<Dependency> {
        if let Ok(file) = std::fs::File::open(self) {
            let file = BufReader::new(file); // increases performance
            let reader = EventReader::new(file);
            let mut group = String::new();
            let mut artifact = String::new();
            let mut version = String::new();
            let mut dependencies: Vec<Dependency> = Vec::new();
            let mut is_dependency = false;
            let mut is_group_id = false;
            let mut is_artifact_id = false;
            let mut is_version = false;

            reader.into_iter().for_each(|element| {
                match &element {
                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("dependency") => {
                        group.clear();
                        artifact.clear();
                        version.clear();
                        is_dependency = true;
                    }
                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("dependency") => {
                        let name = format!("{}.{}", &group, artifact);
                        if let Some(dependency) = Dependency::new(kind, name.as_str(), version.as_str()) {
                            dependencies.push(dependency);
                        }
                        is_dependency = false;
                    }
                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("groupId") => {
                        if is_dependency { is_group_id = true }
                    }
                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("groupId") => {
                        is_group_id = false;
                    }
                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("artifactId") => {
                        if is_dependency { is_artifact_id = true }
                    }
                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("artifactId") => {
                        is_artifact_id = false;
                    }
                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("version") => {
                        if is_dependency { is_version = true }
                    }
                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("version") => {
                        is_version = false;
                    }
                    Ok(XmlEvent::Characters(content)) => {
                        if is_group_id { group = content.clone() }
                        if is_artifact_id { artifact = content.clone() }
                        if is_version { version = content.clone() }
                    }
                    Err(_) => {}
                    _ => {}
                };
            });

            // println!("dependencies: {}", dependencies);

            dependencies
        } else {
            vec![]
        }
    }
}

trait Pom {
    fn to_dependencies(&self, kind: &Kind) -> Vec<Dependency>;
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
                        target_dir: cache.join(relative_path),
                    })
                }
            }
        }
    }
}

struct DependencyInfo {
    pub url: String,
    pub file_suffix: String,
    pub target_dir: PathBuf,
}
