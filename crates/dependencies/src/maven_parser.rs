use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use roxmltree::Node;

use util::sub_strings::SubStrings;

use crate::Parser;

pub struct MavenParser;

impl Parser<crate::Dependency> for MavenParser {
    fn parse(path: PathBuf) -> BTreeSet<crate::Dependency> {
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => return BTreeSet::default(),
        };

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
            .for_each(|it| {
                unique_deps.insert(it);
            });

        managed
            .into_iter()
            .filter(|it| it.version.is_some())
            .for_each(|it| {
                unique_deps.insert(it);
            });

        unique_deps
            .into_iter()
            //.filter(|it| !it.artifact_id.ends_with("-bom"))
            .map(|a| crate::Dependency::from(a))
            .collect()
    }
}

fn interpolate(artifacts: Vec<Artifact>, properties: &HashMap<String, String>) -> Vec<Artifact> {
    artifacts
        .into_iter()
        .map(|mut dep| dep.interpolate(properties))
        .collect::<Vec<_>>()
}

impl From<Artifact> for crate::Dependency {
    fn from(artifact: Artifact) -> Self {
        crate::Dependency::new(
            artifact.group_id,
            artifact.artifact_id,
            artifact.version.unwrap(),
            artifact.scope.into(),
        )
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq)]
pub struct Artifact {
    pub group_id: String,
    pub artifact_id: String,
    pub version: Option<String>,
    pub scope: Scope,
}

impl Artifact {
    fn interpolate(&mut self, properties: &HashMap<String, String>) -> Self {
        if let Some(version) = &self.version {
            if version.contains("${") {
                let property = version
                    .clone()
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

#[derive(Default, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Scope {
    /// default, available at compile-time and runtime
    #[default]
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

impl Into<crate::Kind> for Scope {
    fn into(self) -> crate::Kind {
        match self {
            Scope::Test => crate::Kind::Test,
            _ => crate::Kind::Compile,
        }
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

trait MavenXmlParser {
    fn parse_props(&self) -> HashMap<String, String>;
    fn parse_artifact(&self) -> Artifact;
    fn parse_deps(&self) -> Vec<Artifact>;
    fn parse_managed_deps(&self) -> Vec<Artifact>;
}

/// A property may include a reference to another property
impl MavenXmlParser for Node<'_, '_> {
    fn parse_props(&self) -> HashMap<String, String> {
        match node(self, "properties") {
            Some(properties) => properties
                .children()
                .filter(|node| node.is_element())
                .filter(|node| node.text().is_some())
                .map(|node| {
                    let key = node.tag_name().name().to_owned();
                    let value_or_ref = node.text().expect("property value missing").to_owned();
                    //println!("resolving property: {}:{}", key, value_or_ref);
                    let value = find_property_recursive(properties, value_or_ref)
                        .expect("property value missing");
                    (key, value)
                })
                .collect(),
            _ => HashMap::new(),
        }
    }

    fn parse_artifact(&self) -> Artifact {
        Artifact {
            group_id: node_text(self, "groupId").expect("groupId"),
            artifact_id: node_text(self, "artifactId").expect("artifactId"),
            version: node_text(self, "version"),
            scope: node_text(self, "scope")
                .map(|scope| scope.into())
                .unwrap_or_default(),
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
            _ => vec![],
        }
    }
}

fn find_property_recursive(properties: Node, prop_value: String) -> Option<String> {
    if prop_value.contains("${") {
        let reference = prop_value
            .clone()
            .substr_after('$')
            .remove_surrounding('{', '}');

        if let Some(value) = properties
            .children()
            .find(|n| n.tag_name().name() == reference)
        {
            let value = value.text().expect("property value missing").to_owned();
            return find_property_recursive(properties, value);
        }
    }

    Some(prop_value)

    /*
    let value = properties
        .children()
        .find(|n| {
            let tag = n.tag_name().name().to_owned();
            println!("prop {} == {}", tag, property);
            tag == property
        })
        .map(|n| n.text().unwrap().to_owned())
        .expect("property value missing");

    Some(value)
    */
}

fn node<'a, 'input: 'a>(parent: &'input Node, tag_name: &'a str) -> Option<Node<'a, 'input>> {
    parent
        .children()
        .find(|child| child.is_element() && child.has_tag_name(tag_name))
}

fn node_text<'a, 'input: 'a>(parent: &'input Node, tag_name: &'a str) -> Option<String> {
    let n = node(parent, tag_name)?;
    n.text().map(|t| t.to_owned())
}

#[cfg(test)]
mod tests {
    use crate::{maven_parser::MavenParser, Parser};

    #[test]
    fn dependencies() {
        let pom = home::home_dir()
            .unwrap()
            .join(".buildk/cache")
            .join("org/jetbrains/kotlin/kotlin-stdlib/1.9.22")
            .join("kotlin-stdlib-1.9.22.pom");
        let parsed = MavenParser::parse(pom);

        parsed.iter().for_each(|art| {
            println!("{}:{}:{}", art.group, art.artifact, art.to_owned().version);
        });
    }
}
