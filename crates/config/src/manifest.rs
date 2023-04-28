use std::fmt::{Display, Formatter};
use std::path::Path;

use serde_derive::Deserialize;

use util::get_kotlin_home;

use crate::read_file;
use crate::build::Build;
use crate::dependencies::Dependencies;
use crate::project::Project;

#[derive(Deserialize, Clone)]
pub struct Manifest {
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub dependencies: Dependencies,
    #[serde(default)]
    pub test_dependencies: Dependencies,
}

impl Default for Manifest {
    fn default() -> Self {
        Manifest::new()
    }
}

impl Manifest {
    fn new() -> Manifest {
        let path = manifest_path();
        match read_file(path) {
            Ok(content) => toml::from_str(&content).unwrap(),
            Err(e) => panic!("Failed to build manifest: {}", e)
        }
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        write!(f, "{}", self.build)?;
        writeln!(f, "{:<26}{}", "kotlin.path", get_kotlin_home().display())?;

        self.dependencies.deps.iter().try_for_each(|(name, version)| {
            writeln!(f, "{:<26}{}:{}", "dependency", name, version)
        })?;

        self.test_dependencies.deps.iter().try_for_each(|(name, version)| {
            writeln!(f, "{:<26}{}:{}", "dependency.test", name, version)
        })

    }
}

#[cfg(debug_assertions)]
fn manifest_path() -> &'static Path { Path::new("test/buildk.toml") }

#[cfg(not(debug_assertions))]
fn manifest_path() -> &'static Path { Path::new("buildk.toml") }

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::manifest::Manifest;
    use crate::toml;

    fn manifest(content: &str) -> Manifest {
        toml::<Manifest>(content).unwrap()
    }

    #[test]
    fn default_manifest() {
        let manifest = manifest("");
        assert_eq!(&manifest.project.main_class(), "MainKt");
        assert!(manifest.project.path.ends_with("buildk/crates/config"));
        assert!(manifest.dependencies.deps.is_empty());
        assert_eq!(manifest.build.src, PathBuf::from("src"));
        assert_eq!(manifest.build.test, PathBuf::from("test"));
        assert_eq!(manifest.build.output, PathBuf::from("out"));
        assert_eq!(manifest.build.output_src(), PathBuf::from("out/src"));
        assert_eq!(manifest.build.output_test(), PathBuf::from("out/test"));
        assert_eq!(manifest.build.output_target(), PathBuf::from("out/app.jar"));
        assert_eq!(manifest.build.output_cache(), PathBuf::from("out/cache.json"));
    }

    #[test]
    fn project_main() {
        let manifest = manifest(r#"
[project]
main = "TestMain.kt"
"#);
        assert_eq!(&manifest.project.main_class(), "TestMainKt");
    }

    #[test]
    fn project_path() {
        let manifest = manifest(r#"
[project]
path = "test/dir"
"#);
        assert_eq!(manifest.project.path, PathBuf::from("test/dir"));
    }

    #[test]
    fn build_src() {
        let manifest = manifest(r#"
[build]
src = "awesome/source"
"#);
        assert_eq!(manifest.build.src, PathBuf::from("awesome/source"))
    }

    #[test]
    fn build_test() {
        let manifest = manifest(r#"
[build]
test = "lucky/you"
"#);
        assert_eq!(manifest.build.test, PathBuf::from("lucky/you"))
    }

    #[test]
    fn build_output() {
        let manifest = manifest(r#"
[build]
output = "somewhere/special"
"#);
        assert_eq!(manifest.build.output, PathBuf::from("somewhere/special"))
    }

    #[test]
    fn build_target() {
        let manifest = manifest(r#"
[build]
target = "out/crazy.jar"
"#);
        assert_eq!(manifest.build.target, PathBuf::from("out/crazy.jar"))
    }

    #[test]
    fn build_cache() {
        let manifest = manifest(r#"
[build]
cache = "euphoria"
"#);
        assert_eq!(manifest.build.cache, PathBuf::from("euphoria"))
    }

    #[test]
    fn single_dependency() {
        let manifest = manifest(r#"
[dependencies]
awesome = "1.2.3"
"#);
        assert_eq!(manifest.dependencies.deps.get("awesome"), Some(&"1.2.3".to_string()));
    }

    #[test]
    fn multiple_dependencies() {
        let manifest = manifest(r#"
[dependencies]
awesome = "1.2.3"
amazing = "2.0"
"#);
        assert_eq!(manifest.dependencies.deps.get("awesome"), Some(&"1.2.3".to_string()));
        assert_eq!(manifest.dependencies.deps.get("amazing"), Some(&"2.0".to_string()));
    }


    #[test]
    fn single_test_dependency() {
        let manifest = manifest(r#"
[test_dependencies]
awesome = "1.2.3"
"#);
        assert_eq!(manifest.test_dependencies.deps.get("awesome"), Some(&"1.2.3".to_string()));
    }

    #[test]
    fn multiple_test_dependencies() {
        let manifest = manifest(r#"
[test_dependencies]
awesome = "1.2.3"
amazing = "2.0"
"#);
        assert_eq!(manifest.test_dependencies.deps.get("awesome"), Some(&"1.2.3".to_string()));
        assert_eq!(manifest.test_dependencies.deps.get("amazing"), Some(&"2.0".to_string()));
    }
}
