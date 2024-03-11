use std::collections::{BTreeSet, HashMap};
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;

use roxmltree::Node;

use util::sub_strings::SubStrings;

fn parse_pom(pom: PathBuf) -> BTreeSet<Dependency> {
    let content = fs::read_to_string(pom).unwrap();
    let doc = roxmltree::Document::parse(&content).unwrap();
    let root = doc.root();
    let project = node(&root, "project").expect("invalid pom, missing <project> tag");

    let props = project.parse_props();
    let deps = project.parse_deps();
    let deps = interpolate(deps, &props);
    let managed = project.parse_managed_deps();
    let managed = interpolate(managed, &props);

    let mut unique_deps = BTreeSet::new();

    deps.into_iter()
        .filter(|it| it.version.is_some())
        .map(Dependency::from)
        .for_each(|it| {
            unique_deps.insert(it);
        });

    managed.into_iter()
        .filter(|it| it.version.is_some())
        .map(Dependency::from)
        .for_each(|it| {
            unique_deps.insert(it);
        });

    unique_deps
}

impl From<Artifact> for Dependency {
    fn from(artifact: Artifact) -> Self {
        Dependency {
            kind: artifact.scope.into(),
            name: format!("{}.{}", artifact.group_id, artifact.artifact_id),
            version: artifact.version.unwrap(),
        }
    }
}

fn interpolate(artifacts: Vec<Artifact>, properties: &HashMap<String, String>) -> Vec<Artifact> {
    artifacts.into_iter()
        .map(|mut dep| dep.interpolate(&properties))
        .collect::<Vec<_>>()
}

#[derive(Clone)]
struct Artifact {
    group_id: String,
    artifact_id: String,
    version: Option<String>,
    scope: Scope,
}

impl Artifact {
    fn interpolate(&mut self, properties: &HashMap<String, String>) -> Self {
        if let Some(version) = &self.version {
            if version.contains("${") {
                let property = version.clone()
                    .substr_after('$')
                    .remove_surrounding('{', '}');

                if let Some(version) = properties.get(&property) {
                    self.version = Some(version.to_owned());
                }
            }
        }
        self.clone()
    }
}

#[derive(Clone)]
enum Scope {
    /// default, available at compile-time and runtime
    Compile,
    /// available at compile-time only, but is still required at runtime
    Provided,
    /// only available at runtime
    Runtime,
    /// only available at test-compile-time and test-runtime
    Test,
    /// required at compile-time and runtime, but is not included in the project
    System,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::Compile
    }
}

impl From<String> for Scope {
    fn from(value: String) -> Self {
        match value.as_str() {
            "compile" => Scope::Compile,
            "provided" => Scope::Provided,
            "runtime" => Scope::Runtime,
            "test" => Scope::Test,
            "system" => Scope::System,
            _ => Scope::default(),
        }
    }
}

impl From<Scope> for String {
    fn from(value: Scope) -> Self {
        match value {
            Scope::Runtime => "runtime".to_owned(),
            _ => "compile".to_owned(),
        }
    }
}

trait MavenParser {
    fn parse_props(&self) -> HashMap<String, String>;
    fn parse_artifact(&self) -> Artifact;
    fn parse_deps(&self) -> Vec<Artifact>;
    fn parse_managed_deps(&self) -> Vec<Artifact>;
}

impl MavenParser for Node<'_, '_> {
    fn parse_props(&self) -> HashMap<String, String> {
        match node(self, "properties") {
            Some(properties) => properties
                .children()
                .filter(|node| node.is_element())
                .map(|node| (node.tag_name().name().to_owned(), node.text().unwrap().to_owned()))
                .collect(),
            _ => HashMap::new(),
        }
    }

    fn parse_artifact(&self) -> Artifact {
        Artifact {
            group_id: node_text(self, "groupId").expect("groupId"),
            artifact_id: node_text(self, "artifactId").expect("artifactId"),
            version: node_text(self, "version"),
            scope: node_text(self, "scope").map(|scope| scope.into()).unwrap_or_default(),
        }
    }

    fn parse_deps(&self) -> Vec<Artifact> {
        match node(self, "dependencies") {
            Some(dependencies) => dependencies
                .children()
                .filter(|node| node.is_element() && node.has_tag_name("dependency"))
                .map(|node| node.parse_artifact())
                .collect(),
            _ => vec![],
        }
    }

    fn parse_managed_deps(&self) -> Vec<Artifact> {
        match node(self, "dependencyManagement") {
            Some(node) => node.parse_deps(),
            _ => vec![]
        }
    }
}


fn node<'a, 'input: 'a>(
    parent: &'input Node,
    tag_name: &'a str,
) -> Option<Node<'a, 'input>> {
    parent
        .children()
        .find(|child| child.is_element() && child.has_tag_name(tag_name))
}

fn node_text<'a, 'input: 'a>(parent: &'input Node, tag_name: &'a str) -> Option<String> {
    let n = node(parent, tag_name)?;
    n.text().map(|t| t.to_owned())
}

#[derive(Ord, PartialOrd, Eq, PartialEq)]
struct Dependency {
    kind: String,
    name: String,
    version: String,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{} ({})", self.name, self.version, self.kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependencies() {
        let pom = home::home_dir().unwrap()
            .join(".buildk/cache")
            .join("org/jetbrains/kotlin/kotlin-stdlib/1.9.22")
            .join("kotlin-stdlib-1.9.22.pom");
        let parsed = parse_pom(pom);

        parsed.iter().for_each(|dep| {
            println!("{dep}");
        });
    }
}