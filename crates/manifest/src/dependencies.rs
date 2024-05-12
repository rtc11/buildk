use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Context;
use toml_edit::{Document, Item, Table, Value};

use dependencies::maven_parser::{Artifact, MavenParser, Scope};
use util::sub_strings::SubStrings;

use crate::Section;

// https://docs.gradle.org/current/userguide/dependency_management.html#sec:how-gradle-downloads-deps

#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct Dependency {
    pub name: Name,
    pub version: Version,
    pub kind: Kind,
    pub target_dir: PathBuf,
    pub path: String,
    // url to the artifact directory that contains all the files.
    pub jar: String,
    // Filename
    pub sources: String,
    // Filename
    pub pom: String,
    // Filename
    pub module: String, // Filename
}

#[derive(Copy, Clone, Ord, PartialOrd, Debug, Eq, PartialEq, Hash)]
pub enum Kind {
    Source,
    Test,
    Platform,
    PlatformTest,
}


#[derive(Clone, Ord, PartialOrd, Hash, Eq, PartialEq, Debug)]
pub struct Name(String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Name(value)
    }
}

impl Into<String> for Name {
    fn into(self) -> String {
        self.0
    }
}

impl From<&str> for Name {
    fn from(value: &str) -> Self {
        Name(value.to_string())
    }
}

#[derive(Clone, Ord, PartialOrd, Hash, Eq, PartialEq, Debug)]
pub struct Version(String);

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.name.0, self.version.0)
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Version {
    fn from(value: String) -> Self {
        Version(value)
    }
}

impl From<&str> for Version {
    fn from(value: &str) -> Self {
        Version(value.to_string())
    }
}

pub trait DependenciesTools {
    fn platform_deps(&self) -> Vec<Dependency>;
    fn platform_test_deps(&self) -> Vec<Dependency>;
    fn src_deps(&self) -> Vec<Dependency>;
    fn test_deps(&self) -> Vec<Dependency>;
    fn junit_runner(&self) -> Option<Dependency>;
    fn kotlin_stdlib(&self) -> Option<Dependency>;
    fn junit_platform(&self) -> Option<Dependency>;
}

impl DependenciesTools for Vec<Dependency> {
    fn platform_deps(&self) -> Vec<Dependency> {
        self.iter().filter(|dep| dep.kind == Kind::Platform).cloned().collect()
    }

    fn platform_test_deps(&self) -> Vec<Dependency> {
        self.iter().filter(|dep| dep.kind == Kind::PlatformTest).cloned().collect()
    }

    fn src_deps(&self) -> Vec<Dependency> {
        self.iter().filter(|dep| dep.kind == Kind::Source).cloned().collect()
    }

    fn test_deps(&self) -> Vec<Dependency> {
        self.iter().filter(|dep| dep.kind == Kind::Test).cloned().collect()
    }

    fn junit_runner(&self) -> Option<Dependency> {
        self.platform_test_deps().iter()
            .find(|dep| dep.name.0.eq("org.junit.platform.junit-platform-console-standalone"))
            .cloned()
    }

    fn kotlin_stdlib(&self) -> Option<Dependency> {
        self.platform_deps().iter()
            .find(|dep| dep.name.0.eq("org.jetbrains.kotlin.kotlin-stdlib"))
            .cloned()
    }

    fn junit_platform(&self) -> Option<Dependency> {
        self.platform_test_deps().iter()
            .find(|dep| dep.name.0.eq("org.junit.jupiter.junit-jupiter-api"))
            .cloned()
    }
}

pub(crate) fn create_platform_deps() -> Vec<Dependency> {
    vec![
        Dependency::new(
            Kind::PlatformTest,
            Name::from("org.junit.platform.junit-platform-console-standalone"),
            Version::from("1.10.1"),
        ).unwrap(),
        Dependency::new(
            Kind::Platform,
            Name::from("org.jetbrains.kotlin.kotlin-stdlib"),
            Version::from("1.9.22"),
        ).unwrap(),
        Dependency::new(
            Kind::PlatformTest,
            Name::from("org.junit.jupiter.junit-jupiter-api"),
            Version::from("5.5.2"),
        ).unwrap(),
    ]
}

impl Dependency {
    pub fn classpath(&self) -> String {
        self.transitives()
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

    // todo: if one transitive dep has previously failed, this is not good enough for a check
    pub fn is_cached(&self) -> bool {
        let jar = self.target_dir.join(&self.jar);
        let pom = self.target_dir.join(&self.pom);

        jar.exists()
            && pom.exists()
            && jar.metadata().unwrap().len() > 0
            && pom.metadata().unwrap().len() > 0
    }

    pub fn new(kind: Kind, name: Name, version: Version) -> anyhow::Result<Dependency> {
        let dep = dependency_info(&name, &version)
            .map(|info| Self {
                name,
                version,
                kind,
                target_dir: info.target_dir.clone(),
                path: info.path,
                jar: format!("{}.jar", info.file_suffix),
                sources: format!("{}-sources.jar", info.file_suffix),
                pom: format!("{}.pom", info.file_suffix),
                module: format!("{}.module", info.file_suffix),
            })?;

        Ok(dep)
    }

    pub fn from_toml(kind: Kind, name: &str, item: &Value) -> anyhow::Result<Dependency> {
        let version = item.as_str().context("missing version")?;
        Self::new(kind, Name::from(name), Version::from(version))
    }

    pub fn transitives(&self) -> BTreeSet<Dependency> {
        let pom = self.target_dir.join(&self.pom);
        MavenParser::parse_pom(pom, Dependency::try_from)
    }
}

/// [name] "org.apache.kafka:kafka-clients"
/// [version] "3.4.0"
fn dependency_info(name: &Name, version: &Version) -> anyhow::Result<DependencyInfo> {
    let group_id = resolve_group_id(name)?;
    let artifact_id = resolve_artifact_id(name)?;

    let path = group_id.replace('.', "/");
    let path = format!("{path}/{artifact_id}/{version}/");
    let file_suffix = format!("{artifact_id}-{version}");
    let home = home::home_dir().unwrap().join(".buildk");
    let cache = home.join("cache");
    let target_dir = cache.join(&path);

    Ok(DependencyInfo { path, file_suffix, target_dir })
}

/// [name] org.apache.kafka.kafka-clients   [return] org.apache.kafka
/// [name] org.osgi."org.osgi.core"         [return] org.osgi
/// [name] org.slf4j:slf4j-api              [return] org.slf4j
fn resolve_group_id(name: &Name) -> anyhow::Result<String> {
    let until_first_quote = |name: String| -> String {
        let mut name = name.substr_before('"');
        name.pop().expect("empty string, expected a dot");
        name
    };

    let until_last_dot = |s: String| s.substr_before_last('.');
    let before_colon = |s: String| s.substr_before(':');

    let regex = match &name.0 {
        name if name.contains('"') => until_first_quote,
        name if name.contains(':') => before_colon,
        _ => until_last_dot,
    };

    let group_id = regex(name.clone().0);
    Ok(group_id)
}

/// [name] org.apache.kafka.kafka-clients   [return] kafka-clients
/// [name] org.osgi."org.osgi.core"         [return] org.osgi.core
/// [name] org.slf4j:slf4j-api              [return] slf4j-api
fn resolve_artifact_id(name: &Name) -> anyhow::Result<String> {
    let between_double_quotes = |s: String| s.substr_after('"').substr_before('"');
    let after_last_dot = |s: String| s.substr_after_last('.');
    let after_colon = |s: String| s.substr_after(':');

    let regex = match &name.0 {
        name if name.contains('"') => between_double_quotes,
        name if name.contains(':') => after_colon,
        _ => after_last_dot,
    };

    let artifact_id = regex(name.clone().0);

    Ok(artifact_id)
}

struct DependencyInfo {
    pub path: String,
    pub file_suffix: String,
    pub target_dir: PathBuf,
}

pub trait DependenciesKind {
    fn for_test(&self) -> Vec<&Dependency>;
    fn for_src(&self) -> Vec<&Dependency>;
    fn for_platform(&self) -> Vec<&Dependency>;
}

impl DependenciesKind for Vec<Dependency> {
    fn for_test(&self) -> Vec<&Dependency> {
        self.iter()
            .filter(|dep| dep.kind == Kind::Test)
            .collect()
    }

    fn for_src(&self) -> Vec<&Dependency> {
        self.iter()
            .filter(|dep| dep.kind == Kind::Source)
            .collect()
    }

    fn for_platform(&self) -> Vec<&Dependency> {
        self.iter()
            .filter(|dep| [Kind::Platform, Kind::PlatformTest].contains(&dep.kind))
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

    let platform_deps = create_platform_deps();

    manifested_deps
        .iter()
        .chain(platform_deps.iter())
        .cloned()
        .collect()
}

fn dependencies_for(table: &Table, kind: Kind) -> Vec<Dependency> {
    let mut map = BTreeMap::new();

    table.iter().for_each(|(key, value)| {
        map = decend(map.clone(), vec![key], value);
    });

    map.into_iter()
        .filter_map(|(key, value)| Dependency::from_toml(kind, &key, value).ok())
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

impl TryFrom<Artifact> for Dependency {
    type Error = anyhow::Error;

    fn try_from(value: Artifact) -> Result<Self, Self::Error> {
        Dependency::new(
            value.scope.into(),
            Name::from(format!("{}.{}", value.group_id, value.artifact_id)),
            Version::from(value.version.unwrap()),
        )
    }
}

impl From<Scope> for Kind {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Test => Kind::Test,
            _ => Kind::Source
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait DependencyVersion {
        fn version_for(&self, lib: &str) -> Option<&str>;
    }

    impl DependencyVersion for Vec<Dependency> {
        fn version_for(&self, lib: &str) -> Option<&str> {
            match self.into_iter().find(|dep| dep.name.0.eq(lib)) {
                Some(dependency) => Some(&dependency.version.0),
                None => None,
            }
        }
    }

    trait DependencyKind {
        fn kind_for(&self, lib: &str) -> Option<&Kind>;
    }

    impl DependencyKind for Vec<Dependency> {
        fn kind_for(&self, lib: &str) -> Option<&Kind> {
            match self.into_iter().find(|dep| dep.name.0.eq(lib)) {
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
        splendid.lib = "4.0.0"
        "#
                .parse()
                .unwrap(),
        );

        assert_eq!(dependencies.len(), 4); // +3 platform deps
        assert_eq!(dependencies.version_for("splendid.lib"), Some("4.0.0"));
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

        assert_eq!(dependencies.len(), 4); // +3 platform deps
        assert_eq!(dependencies.version_for("dotted.keys"), Some("1.1"));
    }

    #[test]
    fn multiple_dependencies() {
        let dependencies = dependencies(
            &r#"
[dependencies]
nice.dep = "3.2.1"
amazing.lib = "2.0"
"#
                .parse()
                .unwrap(),
        );

        assert_eq!(dependencies.len(), 5); // +3 platform deps
        assert_eq!(dependencies.version_for("nice.dep"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing.lib"), Some("2.0"));
    }

    #[test]
    fn single_test_dependency() {
        let dependencies = dependencies(
            &r#"
[test-dependencies]
awesome.dep = "1.2.3"
"#
                .parse()
                .unwrap(),
        );

        assert_eq!(dependencies.len(), 4); // +3 platform deps
        assert_eq!(dependencies.version_for("awesome.dep"), Some("1.2.3"));
    }

    #[test]
    fn multiple_test_dependencies() {
        let dependencies = dependencies(
            &r#"
[test-dependencies]
nice.price = "3.2.1"
amazing.ly = "2.0"
"#
                .parse()
                .unwrap(),
        );

        assert_eq!(dependencies.len(), 5); // +3 platform deps
        assert_eq!(dependencies.version_for("nice.price"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing.ly"), Some("2.0"));
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
amazing.a = "2.0"
"#
                .parse()
                .unwrap(),
        );

        assert_eq!(dependencies.len(), 7); // +3 platform deps
        assert_eq!(dependencies.version_for("awesome.lib"), Some("3.0.0"));
        assert_eq!(dependencies.version_for("another.amazing.dep"), Some("2.4"));
        assert_eq!(dependencies.version_for("splendid.test.lib"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing.a"), Some("2.0"));
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

        assert_eq!(dependencies.len(), 5); // +3 platform deps
        assert_eq!(dependencies.version_for("awesome.lib.prod"), Some("3.0.0"));
        assert_eq!(dependencies.version_for("awesome.lib.test"), Some("3.0.1"));
    }

    #[test]
    fn test_dependency_kind() {
        let dependencies = dependencies(
            &r#"
[dependencies]
"awesome.lib" = "3.0.0"

[test-dependencies]
splendid.test.lib = "3.2.1"
"#
                .parse()
                .unwrap(),
        );

        assert_eq!(dependencies.len(), 5); // +3 platform deps
        assert_eq!(dependencies.kind_for("awesome.lib"), Some(&Kind::Source));
        assert_eq!(
            dependencies.kind_for("splendid.test.lib"),
            Some(&Kind::Test)
        );
    }

    #[test]
    fn test_dependency_info() {
        let name = Name::from("org.apache.kafka.kafka-clients");
        let version = Version::from("3.4.0");
        let info = dependency_info(&name, &version).unwrap();
        assert_eq!(info.file_suffix, "kafka-clients-3.4.0");
        assert_eq!(info.path, "org/apache/kafka/kafka-clients/3.4.0/");
        assert_eq!(info.target_dir, PathBuf::from(home::home_dir().unwrap().join(".buildk/cache").join("org/apache/kafka/kafka-clients/3.4.0")));
    }

    #[test]
    fn dep_with_dotted_artifact() {
        let name = Name::from(r#"org.osgi."org.osgi.core""#);
        let version = Version::from("6.0.0");
        let info = dependency_info(&name, &version).unwrap();
        assert_eq!(info.file_suffix, "org.osgi.core-6.0.0");
        assert_eq!(info.path, "org/osgi/org.osgi.core/6.0.0/")
    }

    #[test]
    fn resolve_quoted_group_id() -> anyhow::Result<()> {
        let name = Name::from(r#"org.osgi."org.osgi.core""#);
        let group_id = resolve_group_id(&name)?;
        assert_eq!(group_id, "org.osgi");
        Ok(())
    }

    #[test]
    fn resolve_quoted_artifact_id() -> anyhow::Result<()> {
        let name = Name::from(r#"org.osgi."org.osgi.core""#);
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(artifact_id, "org.osgi.core");
        Ok(())
    }

    #[test]
    fn resolve_dotted_group_id() -> anyhow::Result<()> {
        let name = Name::from("org.apache.kafka.kafka-clients");
        let group_id = resolve_group_id(&name)?;
        assert_eq!(group_id, "org.apache.kafka");
        Ok(())
    }

    #[test]
    fn resolve_dotted_artifact_id() -> anyhow::Result<()> {
        let name = Name::from("org.apache.kafka.kafka-clients");
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(artifact_id, "kafka-clients");
        Ok(())
    }

    #[test]
    fn resolve_colon_group_id() -> anyhow::Result<()> {
        let name = Name::from("org.slf4j:slf4j-api");
        let group_id = resolve_group_id(&name)?;
        assert_eq!(group_id, "org.slf4j");
        Ok(())
    }

    #[test]
    fn resolve_colon_artifact_id() -> anyhow::Result<()> {
        let name = Name::from("org.slf4j:slf4j-api");
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(artifact_id, "slf4j-api");
        Ok(())
    }

    #[test]
    fn test_several_dep_infos() -> anyhow::Result<()> {
        let name = Name::from("org.apache.kafka.kafka-clients");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.apache.kafka");
        assert_eq!(artifact_id, "kafka-clients");

        let name = Name::from("org.junit.platform.junit-platform-console-standalone");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.junit.platform");
        assert_eq!(artifact_id, "junit-platform-console-standalone");

        let name = Name::from("org.jetbrains.kotlin.kotlin-stdlib");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.jetbrains.kotlin");
        assert_eq!(artifact_id, "kotlin-stdlib");

        let name = Name::from("org.junit.jupiter.junit-jupiter-api");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.junit.jupiter");
        assert_eq!(artifact_id, "junit-jupiter-api");

        let name = Name::from("com.github.luben:zstd-jni");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "com.github.luben");
        assert_eq!(artifact_id, "zstd-jni");

        let name = Name::from("org.lz4:lz4-java");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.lz4");
        assert_eq!(artifact_id, "lz4-java");

        let name = Name::from("org.xerial.snappy:snappy-java");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.xerial.snappy");
        assert_eq!(artifact_id, "snappy-java");

        let name = Name::from("org.slf4j:slf4j-api");
        let group_id = resolve_group_id(&name)?;
        let artifact_id = resolve_artifact_id(&name)?;
        assert_eq!(group_id, "org.slf4j");
        assert_eq!(artifact_id, "slf4j-api");

        Ok(())
    }
}
