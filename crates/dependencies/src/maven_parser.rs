use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use roxmltree::Node;

use util::sub_strings::SubStrings;

use crate::Parser;

pub struct MavenParser;

impl MavenParser {
    pub fn parse_pom<FN, OUT: Ord>(pom: PathBuf, from: FN) -> BTreeSet<OUT>
        where FN: Fn(Artifact) -> anyhow::Result<OUT>
    {
        Self::parse(pom)
            .into_iter()
            .filter_map(|artifact| from(artifact).ok())
            .collect()
    }
}

impl Parser<Artifact> for MavenParser {
    fn parse(pom: PathBuf) -> BTreeSet<Artifact> {
        let content = match fs::read_to_string(pom) {
            Ok(content) => content,
            Err(_) => return BTreeSet::default()
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

        managed.into_iter()
            .filter(|it| it.version.is_some())
            .for_each(|it| {
                unique_deps.insert(it);
            });

        unique_deps
    }
}

fn interpolate(artifacts: Vec<Artifact>, properties: &HashMap<String, String>) -> Vec<Artifact> {
    artifacts.into_iter()
        .map(|mut dep| dep.interpolate(properties))
        .collect::<Vec<_>>()
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

impl MavenXmlParser for Node<'_, '_> {
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

#[cfg(test)]
mod tests {
    use crate::maven_parser::MavenParser;

    #[test]
    fn dependencies() {
        let pom = home::home_dir().unwrap()
            .join(".buildk/cache")
            .join("org/jetbrains/kotlin/kotlin-stdlib/1.9.22")
            .join("kotlin-stdlib-1.9.22.pom");
        let parsed = MavenParser::parse_pom(pom, |art|Ok(art) );

        parsed.iter().for_each(|art| {
            println!("{}:{}:{}", art.group_id, art.artifact_id, art.to_owned().version.unwrap());
        });
    }
}