use std::collections::{BTreeMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use std::{io::BufReader, path::PathBuf};
use xml::reader::XmlEvent;
use xml::EventReader;

use crate::buildk;
use crate::Section;
use anyhow::bail;
use regex::Regex;
use toml_edit::{Document, Item, Table, Value};

// https://docs.gradle.org/current/userguide/dependency_management.html#sec:how-gradle-downloads-deps

#[derive(Clone, Debug)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub kind: Kind,
    pub target_dir: PathBuf,
    pub path: String,     // url to the artifact directory that contains all the files.
    pub jar: String,     // Filename
    pub sources: String, // Filename
    pub pom: String,     // Filename
    pub module: String,  // Filename
    classpath: HashSet<Dependency>, // Every transitive dependency
}

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    Source,
    Test,
    Platform,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            Kind::Source => writeln!(f, "{:<26}{}:{}", "dependency", self.name, self.version),
            Kind::Test => writeln!(f, "{:<26}{}:{}", "test-dependency", self.name, self.version),
            Kind::Platform => writeln!(
                f,
                "{:<26}{}:{}",
                "platform-dependency", self.name, self.version
            ),
        }
    }
}

impl Dependency {
    pub fn classpath(&self) -> String {
        self.classpath
            .clone()
            .into_iter()
            .map(|dep| dep.jar_absolute_path())
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .as_slice()
            .join(":")
    }

    pub fn jar_absolute_path(&self) -> PathBuf {
        self.target_dir.join(&self.jar)
    }

    // todo: save to project-cache-file
    // todo: if cache failed, go through every transitive dep (before downloading, check file on
    // disk)
    pub fn is_cached(&self) -> bool {
        self.target_dir.join(&self.jar).is_file()
    }

    pub fn new(kind: &Kind, name: &str, version: &str) -> Option<Dependency> {
        dependency_info(name, version)
            .map(|info| Self {
                name: name.into(),
                version: version.into(),
                kind: kind.clone(),
                target_dir: info.target_dir.clone(),
                path: info.path,
                jar: format!("{}.jar", info.file_suffix),
                sources: format!("{}-sources.jar", info.file_suffix),
                pom: format!("{}.pom", info.file_suffix),
                module: format!("{}.module", info.file_suffix),
                classpath: HashSet::default(),
            })
            .ok()
    }

    pub fn from_toml(
        kind: &Kind,
        name: &str,
        item: &toml_edit::Value,
    ) -> Option<Dependency> {
        item.as_str()
            .and_then(|version| Self::new(kind, name, version))
    }

    pub fn transitives(&self) -> Vec<Dependency> {
        let pom = self.target_dir.join(&self.pom);
        pom.parse_pom(&self.kind)
    }
}

/**
*   [name] "org.apache.kafka.kafka-clients"
*   [version] "3.4.0"
*/
fn dependency_info(name: &str, version: &str) -> anyhow::Result<DependencyInfo> {
    let after_last_slash = Regex::new(r"([^/]+)$").unwrap();
    let name = name.replace('.', "/");
    // todo: place this elsewhere
    let home = buildk::home_dir().unwrap();
    let cache = home.join("cache");

    match after_last_slash.find(&name) {
        None => bail!("artifact not found for dependency"),
        Some(artifact_name) => match name.split('/').map(PathBuf::from).reduce(|a, b| a.join(b)) {
            None => bail!("relative path for dependency not deduced"),
            Some(relative_path) => Ok(DependencyInfo {
                path: format!("{name}/{version}/"),
                file_suffix: format!("{}-{version}", artifact_name.as_str()),
                target_dir: cache.join(relative_path).join(version),
            }),
        },
    }
}

struct DependencyInfo {
    pub path: String,
    pub file_suffix: String,
    pub target_dir: PathBuf,
}

pub trait DependenciesKind {
    fn for_test(self) -> Vec<Dependency>;
    fn for_src(self) -> Vec<Dependency>;
    fn for_platform(self) -> Vec<Dependency>;
}

impl DependenciesKind for Vec<Dependency> {
    fn for_test(self) -> Vec<Dependency> {
        self.into_iter()
            .filter(|dep| dep.kind == Kind::Test)
            .collect()
    }

    fn for_src(self) -> Vec<Dependency> {
        self.into_iter()
            .filter(|dep| dep.kind == Kind::Source)
            .collect()
    }

    fn for_platform(self) -> Vec<Dependency> {
        self.into_iter()
            .filter(|dep| dep.kind == Kind::Platform)
            .collect()
    }
}

pub(crate) fn dependencies(manifest: &Document) -> Vec<Dependency> {
    let manifested_deps = manifest
        .as_table()
        .into_iter()
        .flat_map(|(key, value)| match Section::from_str(key) {
            Ok(Section::Dependencies) => match value.as_table() {
                None => vec![],
                Some(table) => dependencies_for(table, Kind::Source),
            },
            Ok(Section::TestDependencies) => match value.as_table() {
                None => vec![],
                Some(table) => dependencies_for(table, Kind::Test),
            },
            _ => vec![],
        })
        .collect::<Vec<Dependency>>();

    let platform_deps = platform_deps();

    manifested_deps
        .iter()
        .chain(platform_deps.iter())
        .cloned()
        .collect()
}

pub(crate) fn platform_deps() -> Vec<Dependency> {
    vec![
        Dependency::new(
            &Kind::Platform,
            "org.junit.platform.junit-platform-console-standalone",
            "1.10.1",
        )
        .unwrap(),
        Dependency::new(
            &Kind::Platform,
            "org.jetbrains.kotlin.kotlin-stdlib",
            "1.9.22",
        )
        .unwrap(),
    ]
}

fn dependencies_for(table: &Table, kind: Kind) -> Vec<Dependency> {
    let mut map = BTreeMap::new();

    table.iter().for_each(|(key, value)| {
        map = decend(map.clone(), vec![key], value);
    });

    map.into_iter()
        .filter_map(|(key, value)| Dependency::from_toml(&kind, &key, value))
        .collect()
}

/**
 In TOML syntax a dot (.) represents an inline table and not
 part of the field name. This is a workaround to get a list
 of all keys until the value field (that should be the version).
*/
fn decend<'a>(
    mut map: BTreeMap<String, &'a Value>,
    keys: Vec<&'a str>,
    value: &'a Item,
) -> BTreeMap<String, &'a Value> {
    match value {
        Item::Value(value) => {
            map.insert(keys.join("."), value);
        }
        Item::Table(table) => {
            table.iter().for_each(|(key, value)| {
                let mut branching_keys = keys.clone();
                branching_keys.push(key);
                map = decend(map.clone(), branching_keys, value);
            });
        }
        _ => {} // do nothing
    }
    map
}

trait PomParser {
    fn parse_pom(&self, kind: &Kind) -> Vec<Dependency>;
}

impl PomParser for PathBuf {
    fn parse_pom(&self, kind: &Kind) -> Vec<Dependency> {
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
                        if let Some(dependency) =
                            Dependency::new(kind, name.as_str(), version.as_str())
                        {
                            dependencies.push(dependency);
                        }
                        is_dependency = false;
                    }

                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("groupId") => {
                        if is_dependency {
                            is_group_id = true
                        }
                    }

                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("groupId") => {
                        is_group_id = false;
                    }

                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("artifactId") => {
                        if is_dependency {
                            is_artifact_id = true
                        }
                    }

                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("artifactId") => {
                        is_artifact_id = false;
                    }

                    Ok(XmlEvent::StartElement { name, .. }) if name.local_name.eq("version") => {
                        if is_dependency {
                            is_version = true
                        }
                    }

                    Ok(XmlEvent::EndElement { name }) if name.local_name.eq("version") => {
                        is_version = false;
                    }

                    Ok(XmlEvent::Characters(content)) => {
                        if is_group_id {
                            group = content.clone()
                        }
                        if is_artifact_id {
                            artifact = content.clone()
                        }
                        if is_version {
                            version = content.clone()
                        }
                    }

                    Err(_) => {}
                    _ => {}
                };
            });

            dependencies
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Version {
        fn version_for(&self, lib: &str) -> Option<&str>;
    }

    impl Version for Vec<Dependency> {
        fn version_for(&self, lib: &str) -> Option<&str> {
            match self.into_iter().find(|dep| dep.name.eq(lib)) {
                Some(dependency) => Some(&dependency.version),
                None => None,
            }
        }
    }

    trait DependencyKind {
        fn kind_for(&self, lib: &str) -> Option<&Kind>;
    }

    impl DependencyKind for Vec<Dependency> {
        fn kind_for(&self, lib: &str) -> Option<&Kind> {
            match self.into_iter().find(|dep| dep.name.eq(lib)) {
                Some(dependency) => Some(&dependency.kind),
                None => None,
            }
        }
    }

    #[test]
    fn single_dependency() {
        let dependencies = dependencies(
            &r#"
        [dependencies]
        splendid = "4.0.0"
        "#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies.version_for("splendid"), Some("4.0.0"));
    }

    #[test]
    fn dependency_with_dotted_keys() {
        let dependencies = dependencies(
            &r#"
        [dependencies]
        dotted.keys = "1.1"
        "#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies.version_for("dotted.keys"), Some("1.1"));
    }

    #[test]
    fn multiple_dependencies() {
        let dependencies = dependencies(
            &r#"
[dependencies]
nice = "3.2.1"
amazing = "2.0"
"#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.version_for("nice"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
    }

    #[test]
    fn single_test_dependency() {
        let dependencies = dependencies(
            &r#"
[test-dependencies]
awesome = "1.2.3"
"#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies.version_for("awesome"), Some("1.2.3"));
    }

    #[test]
    fn multiple_test_dependencies() {
        let dependencies = dependencies(
            &r#"
[test-dependencies]
nice = "3.2.1"
amazing = "2.0"
"#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.version_for("nice"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
    }

    #[test]
    fn multiple_dependencies_and_test_dependencies() {
        let dependencies = dependencies(
            &r#"
[dependencies]
awesome.lib = "3.0.0"
another.amazing.dep = "2.4"

[test-dependencies]
splendid.test.lib = "3.2.1"
amazing = "2.0"
"#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies.version_for("awesome.lib"), Some("3.0.0"));
        assert_eq!(dependencies.version_for("another.amazing.dep"), Some("2.4"));
        assert_eq!(dependencies.version_for("splendid.test.lib"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
    }

    #[test]
    fn multiple_dependencies_within_same_namespace() {
        let dependencies = dependencies(
            &r#"
[dependencies]
awesome.lib.prod = "3.0.0"
awesome.lib.test = "3.0.1"

"#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.version_for("awesome.lib.prod"), Some("3.0.0"));
        assert_eq!(dependencies.version_for("awesome.lib.test"), Some("3.0.1"));
    }

    #[test]
    fn test_dependency_kind() {
        let dependencies = dependencies(
            &r#"
[dependencies]
awesome.lib = "3.0.0"

[test-dependencies]
splendid.test.lib = "3.2.1"
"#
            .parse()
            .unwrap(),
        );

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.kind_for("awesome.lib"), Some(&Kind::Source));
        assert_eq!(
            dependencies.kind_for("splendid.test.lib"),
            Some(&Kind::Test)
        );
    }
}
