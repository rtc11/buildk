use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

use anyhow::Context;

use util::get_kotlin_home;

use crate::dependencies::{dependencies, Dependency};
use crate::module::Module;
use crate::project::{Project, project};
use crate::read_file;

pub struct Manifest {
    pub project: Project,
    pub modules: Vec<Module>,
    pub dependencies: Vec<Dependency>,
}

impl Default for Manifest {
    fn default() -> Self {
        let content = read_file(manifest_path()).unwrap();
        let toml = TomlParser::from_str(&content).unwrap();

        Manifest {
            project: toml.project().unwrap_or_default(),
            modules: toml.modules(),
            dependencies: toml.dependencies(),
        }
    }
}

pub struct TomlParser {
    data: toml_edit::Document,
}

pub enum Section {
    Project,
    Module,
    Dependencies,
    TestDependencies,
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "project" => Section::Project,
            "module" => Section::Module,
            "dependencies" => Section::Dependencies,
            "test-dependencies" => Section::TestDependencies,
            _ => anyhow::bail!("Invalid section: {}", s),
        })
    }
}

impl TomlParser {
    pub fn project(&self) -> Option<Project> {
        project(&self.data)
    }

    pub fn modules(&self) -> Vec<Module> {
        vec![]
    }

    pub fn dependencies(&self) -> Vec<Dependency> {
        dependencies(&self.data)
    }
}

impl FromStr for TomlParser {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let manifest: toml_edit::Document = s.parse().context("Manifest not valid TOML.")?;
        Ok(TomlParser { data: manifest })
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        writeln!(f, "{:<26}{}", "kotlin.path", get_kotlin_home().display())?;
        self.dependencies.iter().try_for_each(|dependency| write!(f, "{}", dependency))
    }
}

#[cfg(debug_assertions)]
fn manifest_path() -> &'static Path { Path::new("test/buildk.toml") }

#[cfg(not(debug_assertions))]
fn manifest_path() -> &'static Path { Path::new("buildk.toml") }

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::dependencies::{Dependency, Kind};
    use crate::manifest::TomlParser;

//
//     fn manifest(content: &str) -> Manifest {
//         toml::<Manifest>(content).unwrap()
//     }
//
//     #[test]
//     fn default_manifest() {
//         let manifest = manifest("");
//         assert_eq!(&manifest.project.main_class(), "MainKt");
//         assert!(manifest.project.path.ends_with("buildk/crates/config"));
//         assert_eq!(manifest.build.src, PathBuf::from("src"));
//         assert_eq!(manifest.build.test, PathBuf::from("test"));
//         assert_eq!(manifest.build.output, PathBuf::from("out"));
//         assert_eq!(manifest.build.output_src(), PathBuf::from("out/src"));
//         assert_eq!(manifest.build.output_test(), PathBuf::from("out/test"));
//         assert_eq!(manifest.build.output_target(), PathBuf::from("out/app.jar"));
//         assert_eq!(manifest.build.output_cache(), PathBuf::from("out/cache.json"));
//     }
//
//     #[test]
//     fn project_main() {
//         let manifest = manifest(r#"
// [project]
// main = "TestMain.kt"
// "#);
//         assert_eq!(&manifest.project.main_class(), "TestMainKt");
//     }
//
//     #[test]
//     fn project_path() {
//         let manifest = manifest(r#"
// [project]
// path = "test/dir"
// "#);
//         assert_eq!(manifest.project.path, PathBuf::from("test/dir"));
//     }
//
//     #[test]
//     fn build_src() {
//         let manifest = manifest(r#"
// [build]
// src = "awesome/source"
// "#);
//         assert_eq!(manifest.build.src, PathBuf::from("awesome/source"))
//     }
//
//     #[test]
//     fn build_test() {
//         let manifest = manifest(r#"
// [build]
// test = "lucky/you"
// "#);
//         assert_eq!(manifest.build.test, PathBuf::from("lucky/you"))
//     }
//
//     #[test]
//     fn build_output() {
//         let manifest = manifest(r#"
// [build]
// output = "somewhere/special"
// "#);
//         assert_eq!(manifest.build.output, PathBuf::from("somewhere/special"))
//     }
//
//     #[test]
//     fn build_target() {
//         let manifest = manifest(r#"
// [build]
// target = "out/crazy.jar"
// "#);
//         assert_eq!(manifest.build.target, PathBuf::from("out/crazy.jar"))
//     }
//
//     #[test]
//     fn build_cache() {
//         let manifest = manifest(r#"
// [build]
// cache = "euphoria"
// "#);
//         assert_eq!(manifest.build.cache, PathBuf::from("euphoria"))
//     }

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
