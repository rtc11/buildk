use std::str::FromStr;
use std::{collections::BTreeMap, fmt::Display};

use anyhow::Context;
use dependency::{Package, PackageKind};
use toml_edit::{DocumentMut, Item, Table, Value};

use crate::Section;

// https://docs.gradle.org/current/userguide/dependency_management.html#sec:how-gradle-downloads-deps

#[derive(Clone)]
pub struct Packages {
    pub pkgs: Vec<Package>, // TODO: can we make this private?
}

impl Display for Packages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pkgs = self
            .pkgs
            .iter()
            .map(|pkg| format!("{pkg}"))
            .collect::<Vec<_>>()
            .join(":");

        write!(f, "{}", pkgs)
    }
}

impl Packages {
    pub(crate) fn new(pkgs: Vec<Package>) -> Self {
        Packages { pkgs }
    }

    pub(crate) fn compile(&self) -> Vec<Package> {
        self.pkgs
            .clone()
            .into_iter()
            .filter(|pkg| pkg.kind == PackageKind::Compile)
            .collect()
    }

    pub(crate) fn runtime(&self) -> Vec<Package> {
        self.pkgs
            .clone()
            .into_iter()
            .filter(|pkg| pkg.kind == PackageKind::Runtime)
            .collect()
    }

    pub(crate) fn test(&self) -> Vec<Package> {
        self.pkgs
            .clone()
            .into_iter()
            .filter(|pkg| pkg.kind == PackageKind::Test)
            .collect()
    }

    pub fn filter_cached(&self) -> Vec<Package> {
        self.pkgs
            .iter()
            .filter(|pkg| pkg.is_cached())
            .cloned()
            .collect()
    }
}

pub(crate) fn provided_pkgs() -> Vec<Package> {
    vec![
        Package::new(
            "kotlin-stdlib".to_string(),
            Some("org.jetbrains.kotlin".to_string()),
            "2.0.0".to_string(),
            PackageKind::Compile,
        ),
        // Package::new(
        //     "kotlin-test".to_string(),
        //     Some("org.jetbrains.kotlin".to_string()),
        //     "1.9.22".to_string(),
        //     PackageKind::Test,
        // ),
        Package::new(
            "kotlin-test-junit5".to_string(),
            Some("org.jetbrains.kotlin".to_string()),
            "2.0.0".to_string(),
            PackageKind::Test,
        ),
        Package::new(
            "junit-platform-console-standalone".to_string(),
            Some("org.junit.platform".to_string()),
            "1.10.2".to_string(),
            PackageKind::Test,
        ),
        Package::new(
            "junit-jupiter-api".to_string(),
            Some("org.junit.jupiter".to_string()),
            "5.5.2".to_string(),
            PackageKind::Test,
        ),
    ]
}

/* impl Package {
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

    pub fn transitives(&self) -> BTreeSet<Package> {

        dependencies::resolve(&self.target_dir.join(&self.jar))
            .into_iter()
            .map(|dep| Dependency::try_from(dep).unwrap())
            .collect()
    }

    pub fn jar_absolute_path(&self) -> PathBuf {
        self.target_dir.join(&self.jar)
    }

    // todo: if one transitive dep has previously failed, this is not good enough for a check
    pub fn is_cached(&self) -> bool {
        let jar = self.target_dir.join(&self.jar);

        // TODO check that at least one file descriptor exists with length > 0
        jar.exists() && jar.metadata().unwrap().len() > 0
        //&& pom.exists()
        //&& pom.metadata().unwrap().len() > 0
    }

    pub fn from_toml(kind: Kind, name: &str, item: &Value) -> anyhow::Result<Package> {
        let version = item.as_str().context("missing version")?;
        Self::new(kind, Name::from(name), Version::from(version))
    }

} */

/// [name] "org.apache.kafka:kafka-clients"
/// [version] "3.4.0"
/* fn dependency_info(name: &Name, version: &Version) -> anyhow::Result<DependencyInfo> {
    let group_id = resolve_group_id(name)?;
    let artifact_id = resolve_artifact_id(name)?;

    let path = group_id.replace('.', "/");
    let path = format!("{path}/{artifact_id}/{version}/");
    let file_prefix = format!("{artifact_id}-{version}");
    let home = home::home_dir().unwrap().join(".buildk");
    let cache = home.join("cache");
    let target_dir = cache.join(&path);

    Ok(DependencyInfo {
        path,
        file_prefix,
        target_dir,
    })
} */

/// [name] org.apache.kafka.kafka-clients   [return] org.apache.kafka
/// [name] org.osgi."org.osgi.core"         [return] org.osgi
/// [name] org.slf4j:slf4j-api              [return] org.slf4j
/* fn resolve_group_id(name: &Name) -> anyhow::Result<String> {
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
} */

/// [name] org.apache.kafka.kafka-clients   [return] kafka-clients
/// [name] org.osgi."org.osgi.core"         [return] org.osgi.core
/// [name] org.slf4j:slf4j-api              [return] slf4j-api
/* fn resolve_artifact_id(name: &Name) -> anyhow::Result<String> {
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
*/

/* struct DependencyInfo {
    pub path: String,
    pub file_prefix: String,
    pub target_dir: PathBuf,
}  */

impl From<&DocumentMut> for Packages {
    fn from(toml: &DocumentMut) -> Self {
        let pkgs = dependencies(toml);
        Packages { pkgs }
    }
}

fn dependencies(manifest: &DocumentMut) -> Vec<Package> {
    let manifested_deps = manifest
        .as_table()
        .into_iter()
        .flat_map(|(key, value)| match Section::from_str(key) {
            Ok(Section::CompileDeps) => match value.as_table() {
                None => vec![],
                Some(table) => dependencies_for(table, PackageKind::Compile),
            },
            Ok(Section::RuntimeDeps) => match value.as_table() {
                None => vec![],
                Some(table) => dependencies_for(table, PackageKind::Runtime),
            },
            Ok(Section::TestDeps) => match value.as_table() {
                None => vec![],
                Some(table) => dependencies_for(table, PackageKind::Test),
            },
            _ => vec![],
        })
        .collect::<Vec<Package>>();

    let provided = provided_pkgs();

    manifested_deps
        .iter()
        .chain(provided.iter())
        .cloned()
        .collect()
}

fn dependencies_for(table: &Table, kind: PackageKind) -> Vec<Package> {
    let mut map = BTreeMap::new();

    table.iter().for_each(|(key, value)| {
        map = decend(map.clone(), vec![key], value);
    });

    map.into_iter()
        .filter_map(|(key, value)| pkg_from_toml(kind.clone(), &key, value).ok())
        .collect()
}

fn pkg_from_toml(kind: PackageKind, name: &str, item: &Value) -> anyhow::Result<Package> {
    let version = item.as_str().context("missing version")?.to_string();

    let artifact = name.split("_").collect::<Vec<&str>>();
    let (name, namespace) = match artifact.len() {
        1 => {
            let name = artifact[0].to_string();
            (name, None)
        }
        2 => {
            let name = artifact[1].to_string();
            let namespace = artifact[0].to_string();
            (name, Some(namespace))
        }
        _ => panic!("unexpected artifact name"),
    };

    Ok(Package::new(name, namespace, version, kind))
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
/*
impl TryFrom<dependencies::Dependency> for Dependency {
    type Error = anyhow::Error;

    fn try_from(value: dependencies::Dependency) -> Result<Self, Self::Error> {
        Dependency::new(
            value.kind.into(),
            Name::from(format!("{}.{}", value.group, value.artifact)),
            Version::from(value.version),
        )
    }
}

impl From<dependencies::Kind> for Kind {
    fn from(value: dependencies::Kind) -> Self {
        match value {
            dependencies::Kind::Compile => Kind::Source,
            dependencies::Kind::Test => Kind::Test,
            dependencies::Kind::Runtime => Kind::Source,
            dependencies::Kind::Transparent => Kind::Source,
            dependencies::Kind::Internal => Kind::Platform,
        }
    }
}
*/
/* #[cfg(test)]
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
        assert_eq!(info.file_prefix, "kafka-clients-3.4.0");
        assert_eq!(info.path, "org/apache/kafka/kafka-clients/3.4.0/");
        assert_eq!(
            info.target_dir,
            PathBuf::from(
                home::home_dir()
                    .unwrap()
                    .join(".buildk/cache")
                    .join("org/apache/kafka/kafka-clients/3.4.0")
            )
        );
    }

    #[test]
    fn dep_with_dotted_artifact() {
        let name = Name::from(r#"org.osgi."org.osgi.core""#);
        let version = Version::from("6.0.0");
        let info = dependency_info(&name, &version).unwrap();
        assert_eq!(info.file_prefix, "org.osgi.core-6.0.0");
        assert_eq!(info.path, "org/osgi/org.osgi.core/6.0.0/")
    }

    #[test]
    fn dep_with_dotted_artifact2() {
        let name = Name::from(r#"org.osgi.osgi.core"#);
        let version = Version::from("6.0.0");
        let info = dependency_info(&name, &version).unwrap();
        assert_eq!(info.file_prefix, "org.osgi.core-6.0.0");
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
} */
