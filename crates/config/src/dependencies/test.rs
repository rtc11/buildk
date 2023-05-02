use crate::dependencies::{dependencies, Dependency};
use crate::dependencies::kind::Kind;


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
    let dependencies = dependencies(&r#"
[dependencies]
splendid = "4.0.0"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies.version_for("splendid"), Some("4.0.0"));
}

#[test]
fn dependency_with_dotted_keys() {
    let dependencies = dependencies(&r#"
[dependencies]
dotted.keys = "1.1"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies.version_for("dotted.keys"), Some("1.1"));
}

#[test]
fn multiple_dependencies() {
    let dependencies = dependencies(&r#"
[dependencies]
nice = "3.2.1"
amazing = "2.0"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies.version_for("nice"), Some("3.2.1"));
    assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
}

#[test]
fn single_test_dependency() {
    let dependencies = dependencies(&r#"
[test-dependencies]
awesome = "1.2.3"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies.version_for("awesome"), Some("1.2.3"));
}

#[test]
fn multiple_test_dependencies() {
    let dependencies = dependencies(&r#"
[test-dependencies]
nice = "3.2.1"
amazing = "2.0"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies.version_for("nice"), Some("3.2.1"));
    assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
}

#[test]
fn multiple_dependencies_and_test_dependencies() {
    let dependencies = dependencies(&r#"
[dependencies]
awesome.lib = "3.0.0"
another.amazing.dep = "2.4"

[test-dependencies]
splendid.test.lib = "3.2.1"
amazing = "2.0"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies.version_for("awesome.lib"), Some("3.0.0"));
    assert_eq!(dependencies.version_for("another.amazing.dep"), Some("2.4"));
    assert_eq!(dependencies.version_for("splendid.test.lib"), Some("3.2.1"));
    assert_eq!(dependencies.version_for("amazing"), Some("2.0"));
}

#[test]
fn test_dependency_kind() {
    let dependencies = dependencies(&r#"
[dependencies]
awesome.lib = "3.0.0"
[test-dependencies]
splendid.test.lib = "3.2.1"
"#.parse().unwrap());

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies.kind_for("awesome.lib"), Some(&Kind::Production));
    assert_eq!(dependencies.kind_for("splendid.test.lib"), Some(&Kind::Test));
}
