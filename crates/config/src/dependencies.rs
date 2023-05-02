use std::fmt::{Debug, Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::{Document, Item, Table};
use crate::manifest::Section;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    PRODUCTION,
    TEST,
}

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
            Kind::PRODUCTION => writeln!(f, "{:<26}{}:{}", "dependency", self.name, self.version),
            Kind::TEST => writeln!(f, "{:<26}{}:{}", "test-dependency", self.name, self.version)
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

    fn from_toml(kind: Kind, name: &str, item: &toml_edit::Item) -> anyhow::Result<Dependency> {
        if let Some(version) = item.as_str() {
            let dependency = Self::new(kind, name, version);
            Ok(dependency)
        } else {
            anyhow::bail!("Unresolved dependency, kind: {:?}, name: {name}, version: {item}", kind)
        }
    }
}

pub fn dependencies(data: &Document) -> Vec<Dependency> {
    data.as_table().into_iter().flat_map(|(key, value)| {
        match Section::from_str(key) {
            Ok(Section::Dependencies) =>
                match value.as_table() {
                    None => vec![],
                    Some(table) => dependencies_for(table, Kind::PRODUCTION)
                }
            Ok(Section::TestDependencies) =>
                match value.as_table() {
                    None => vec![],
                    Some(table) => dependencies_for(table, Kind::TEST)
                }
            _ => vec![]
        }
    }).collect::<Vec<Dependency>>()
}

fn dependencies_for(table: &Table, kind: Kind) -> Vec<Dependency> {
    table.iter().flat_map(|(key, item)| {
        let (traversed_keys, item) = traverse_decending_keys(vec![key], item);
        let dependency_name = traversed_keys.join(".");
        match Dependency::from_toml(kind.clone(), &dependency_name, item) {
            Ok(dependency) => Some(dependency),
            Err(_) => None
        }
    }).collect()
}

/// In TOML syntax a dot (.) represents an inline table and not part of the field name.
/// This is a workaround to get a list of all keys until the value field (that should be the version).
fn traverse_decending_keys<'a>(mut keys: Vec<&'a str>, item: &'a Item) -> (Vec<&'a str>, &'a Item) {
    match item {
        Item::Table(table) => {
            match table.len() {
                0 => (keys, item),
                1 => {
                    let (next_key, next_item) = table.iter().last().unwrap();
                    keys.push(next_key);
                    traverse_decending_keys(keys, next_item)
                }
                _ => panic!(r#"Unsupported dependency syntax. A dependency should look like: a.b.c = "1.2.3""#),
            }
        }
        Item::Value(_) => (keys, item),
        _ => panic!(r#"Unsupported dependency syntax. A dependency should look like: a.b.c = "1.2.3""#),
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::dependencies::{Dependency, Kind};
    use crate::manifest::TomlParser;

    #[test]
    fn single_dependency() {
        let manifest = TomlParser::from_str(r#"
[dependencies]
splendid = "4.0.0"
"#).unwrap();

        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies.version_for("splendid"), Some("4.0.0"));
    }

    #[test]
    fn dependency_with_dotted_keys() {
        let manifest = TomlParser::from_str(r#"
[dependencies]
dotted.keys = "1.1"
"#).unwrap();

        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies.version_for("dotted.keys"), Some("1.1"));
    }

    #[test]
    fn multiple_dependencies() {
        let manifest = TomlParser::from_str(r#"
[dependencies]
nice = "3.2.1"
amazing = "2.0"
"#).unwrap();

        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.version_for("nice"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
    }

    #[test]
    fn single_test_dependency() {
        let manifest = TomlParser::from_str(r#"
[test-dependencies]
awesome = "1.2.3"
"#).unwrap();
        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies.version_for("awesome"), Some("1.2.3"));
    }

    #[test]
    fn multiple_test_dependencies() {
        let manifest = TomlParser::from_str(r#"
[test-dependencies]
nice = "3.2.1"
amazing = "2.0"
"#).unwrap();

        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.version_for("nice"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
    }

    #[test]
    fn multiple_dependencies_and_test_dependencies() {
        let manifest = TomlParser::from_str(r#"
[dependencies]
awesome.lib = "3.0.0"
another.amazing.dep = "2.4"

[test-dependencies]
splendid.test.lib = "3.2.1"
amazing = "2.0"
"#).unwrap();

        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies.version_for("awesome.lib"), Some("3.0.0"));
        assert_eq!(dependencies.version_for("another.amazing.dep"), Some("2.4"));
        assert_eq!(dependencies.version_for("splendid.test.lib"), Some("3.2.1"));
        assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
    }

    #[test]
    fn test_dependency_kind() {
        let manifest = TomlParser::from_str(r#"
[dependencies]
awesome.lib = "3.0.0"
[test-dependencies]
splendid.test.lib = "3.2.1"
"#).unwrap();

        let dependencies = manifest.dependencies();
        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies.kind_for("awesome.lib"), Some(&Kind::PRODUCTION));
        assert_eq!(dependencies.kind_for("splendid.test.lib"), Some(&Kind::TEST));
    }

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
}
