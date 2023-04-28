use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::Path;

use serde_derive::Deserialize;

use util::get_kotlin_home;

use crate::{read_file, toml};
use crate::build::Build;
use crate::dependency::Dependency;
use crate::project::Project;

#[derive(Clone)]
pub struct Manifest {
    pub project: Project,
    pub build: Build,
    pub dependencies: Vec<Dependency>,
    pub test_dependencies: Vec<Dependency>,
}

impl Default for Manifest {
    fn default() -> Self {
        match Manifest::new() {
            Ok(manifest) => manifest,
            Err(e) => panic!("Failed to build manifest: {}", e)
        }
    }
}

#[derive(Deserialize)]
struct InternalManifest {
    project: Option<Project>,
    build: Option<Build>,
    dependency: Option<HashMap<String, String>>,
    #[serde(rename = "test.dependency")]
    test_dependency: Option<HashMap<String, String>>,
}

impl Manifest {
    fn new() -> anyhow::Result<Self> {
        let content = read_file(manifest_path())?;
        let internal_manifest = toml::<InternalManifest>(content)?;
        internal_manifest.try_into()
    }
}

trait IntoDependencyVec: Sized {
    fn into_dependency_vec(self, is_test: bool) -> Vec<Dependency>;
}

impl IntoDependencyVec for Option<HashMap<String, String>> {
    fn into_dependency_vec(self, test: bool) -> Vec<Dependency> {
        match self {
            None => vec![],
            Some(dependencies) => dependencies.into_iter()
                .map(|(name, version)| Dependency { name, version, test })
                .collect()
        }
    }
}

impl TryInto<Manifest> for InternalManifest {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Manifest, Self::Error> {
        Ok(Manifest {
            project: self.project.unwrap_or_default(),
            build: self.build.unwrap_or_default(),
            dependencies: self.dependency.into_dependency_vec(false),
            test_dependencies: self.test_dependency.into_dependency_vec(true),
        })
    }
}

impl Display for Manifest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.project)?;
        write!(f, "{}", self.build)?;
        writeln!(f, "{:<26}{}", "kotlin.path", get_kotlin_home().display())?;

        let mut dependencies = String::new();
        self.dependencies.iter().for_each(|dep| {
            dependencies.push_str(&format!("{dep}"))
        });
        self.test_dependencies.iter().for_each(|dep| {
            dependencies.push_str(&format!("{dep}"))
        });
        write!(f, "{}", dependencies)
    }
}

#[cfg(debug_assertions)]
fn manifest_path() -> &'static Path { Path::new("test/buildk.toml") }

#[cfg(not(debug_assertions))]
fn manifest_path() -> &'static Path { Path::new("buildk.toml") }

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::dependency::Dependency;
    use crate::manifest::{InternalManifest, Manifest};
    use crate::toml;

    fn manifest(content: &str) -> Manifest {
        toml::<InternalManifest>(content.to_string()).unwrap().try_into().unwrap()
    }

    #[test]
    fn default_manifest() {
        let manifest = manifest("");
        assert_eq!(&manifest.project.main_class(), "MainKt");
        assert!(manifest.project.path.ends_with("buildk/crates/config"));
        assert!(manifest.dependencies.is_empty());
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
[dependency]
awesome = "1.2.3"
"#);
        let expected = Dependency {
            name: "awesome".to_string(),
            version: "1.2.3".to_string(),
            test: false,
        };
        assert_eq!(manifest.dependencies.first(), Some(&expected))
    }

    #[test]
    fn multiple_dependencies() {
        let manifest = manifest(r#"
[dependency]
awesome = "1.2.3"
amazing = "2.0"
"#);
        let expected = vec![
            Dependency {
                name: "awesome".to_string(),
                version: "1.2.3".to_string(),
                test: false,
            },
            Dependency {
                name: "amazing".to_string(),
                version: "2.0".to_string(),
                test: false,
            },
        ];
        assert_eq!(manifest.dependencies, expected)
    }
}
