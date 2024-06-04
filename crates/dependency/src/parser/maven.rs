use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::PathBuf;

use roxmltree::Node;

use util::sub_strings::SubStrings;

use crate::{Package, PackageKind, Parser};

pub struct MavenParser;

impl Parser<Package> for MavenParser {
    fn parse(path: PathBuf, kind: PackageKind) -> BTreeSet<Package> {
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
            .filter(|it| kind == it.scope.clone().into())
            .for_each(|it| {
                unique_deps.insert(it);
            });

        managed
            .into_iter()
            .filter(|it| it.version.is_some())
            .filter(|it| kind == it.scope.clone().into())
            .for_each(|it| {
                unique_deps.insert(it);
            });

        unique_deps.into_iter().map(Package::from).collect()
    }
}

// we need an internal state to keep track of the properties
#[derive(Ord, PartialOrd, Eq, PartialEq, Default, Debug, Clone)]
pub struct Artifact {
    group: String,
    artifact: String,
    version: Option<String>, // version may be a property, managed (defined elsewhere), or direct
    scope: String,
    location: PathBuf,
}

impl From<Artifact> for Package {
    fn from(artifact: Artifact) -> Self {
        Package::new(
            artifact.artifact,
            Some(artifact.group),
            artifact.version.expect("version missing"),
            artifact.scope.into(),
        )
    }
}

fn interpolate(artifacts: Vec<Artifact>, properties: &HashMap<String, String>) -> Vec<Artifact> {
    artifacts
        .into_iter()
        .map(|mut dep| dep.interpolate(properties))
        .collect::<Vec<_>>()
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

impl From<String> for PackageKind {
    fn from(value: String) -> Self {
        // TODO: provided scopes must be handled differently
        // If a package is provided, it must be added to a set of provided packages,
        // where a package higher up in the hierachy (closer to the manifested packages) are
        // defining it. Here the provided package is exposing its true scope. If missing, the
        // buildsystem should notify the user that this package must be included as either
        // compile, runtime or test in the manifest
        match value.as_str() {
            "compile" => PackageKind::Compile,
            "provided" => PackageKind::Runtime,
            "runtime" => PackageKind::Runtime,
            "test" => PackageKind::Test,
            "system" => PackageKind::Compile,
            _ => PackageKind::default(),
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
            group: node_text(self, "groupId").expect("group_id"),
            artifact: node_text(self, "artifactId").expect("artifact_id"),
            version: node_text(self, "version"),
            scope: node_text(self, "scope").unwrap_or_else(|| "compile".to_owned()),
            ..Default::default()
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
    use crate::{parser::maven::MavenParser, Parser};

    #[test]
    fn transitive_pkgs() {
        let pom = home::home_dir()
            .unwrap()
            .join(".buildk/cache")
            .join("org/jetbrains/kotlin/kotlin-stdlib/1.9.22")
            .join("kotlin-stdlib-1.9.22.pom");
        let pkgs = MavenParser::parse(pom);

        pkgs.iter().for_each(|pkg| {
            println!("name: {}", pkg.name);
            println!("namespace: {}", pkg.namespace.clone().unwrap());
            println!("version: {}", pkg.version);
        });
    }
}
