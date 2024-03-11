#![allow(dead_code)]

use std::{collections::HashMap, fmt::{Display, Formatter}};

use anyhow::{Context, Result};
use async_std::task;
use roxmltree::Node;

use util::sub_strings::SubStrings;

use crate::{Dependency, DepGraph, DepGraphParser, Scope};

pub struct MavenParser;

impl DepGraphParser for MavenParser {
    fn parse(&self, input: String) -> Result<DepGraph> {
        let doc = roxmltree::Document::parse(&input).unwrap();
        let root_node = doc.root();
        let project_node = node(&root_node, "project").context("invalid pom, missing <project> tag")?;

        let project = Project::new(project_node);

        let mut graph = DepGraph::default();
        let root_idx = graph.add(project.artifact.into());

        project.dependencies.iter().for_each(|dep| {
            let idx = graph.add(dep.clone().into());
            graph.connect(root_idx, idx);
        });

        Ok(graph)
    }
}

/*
impl Into<Dependency> for Artifact {
    fn into(self) -> Dependency {
        Dependency {
            group: self.group_id.unwrap_or("unknown".into()),
            artifact: self.artifact_id.unwrap_or("unknown".into()),
            version: self.version.unwrap_or("unknown".into()),
            scope: self.scope.unwrap_or_default().into(),
        }
    }
}
*/

#[derive(Clone)]
struct Project {
    artifact: Artifact,
    parent: Option<Parent>,
    dependency_management: Vec<Artifact>,
    dependencies: Vec<Artifact>,
    properties: HashMap<String, String>,
}

impl Project {
    pub fn new(node: Node) -> Self {
        let artifact = parse_artifact(&node);
        let parent = parse_parent(&node);
        let properties = parse_properties(&node);

        let dependencies = interpolate(parse_dependencies(&node), &properties);
        let dependency_management = interpolate(parse_dependency_management(&node), &properties);

        Self {
            artifact,
            parent,
            dependency_management,
            dependencies,
            properties,
        }
    }
}

fn interpolate(artifacts: Vec<Artifact>, properties: &HashMap<String, String>) -> Vec<Artifact> {
    artifacts.into_iter()
        .map(|mut dep| dep.interpolate_version(&properties))
        .collect::<Vec<_>>()
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Hash)]
struct Artifact {
    group_id: Option<String>,
    artifact_id: Option<String>,
    version: Option<String>,

    scope: Option<MavenScope>,

    // maven = pom, jar, war, ear, ejb, rar, par
    packaging: Option<Packaging>,

    classifier: Option<Classifier>,
}

impl From<Artifact> for Dependency {
    fn from(artifact: Artifact) -> Self {
        Dependency {
            group: artifact.group_id.unwrap_or("unknown".into()),
            artifact: artifact.artifact_id.unwrap_or("unknown".into()),
            version: artifact.version.unwrap_or("unknown".into()),
            scope: artifact.scope.unwrap_or_default().into(),
        }
    }
}

// https://maven.apache.org/ref/3.8.4/maven-core/artifact-handlers.html
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Classifier {
    Jdk(String),
    Sources,
    Javadoc,
    Unknown(String),
}

impl Display for Classifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Classifier::Jdk(v) => write!(f, "{}", v),
            Classifier::Sources => write!(f, "sources"),
            Classifier::Javadoc => write!(f, "javadoc"),
            Classifier::Unknown(v) => write!(f, "{}", v),
        }
    }
}

impl From<String> for Classifier {
    fn from(value: String) -> Self {
        match value.as_str() {
            "sources" => Classifier::Sources,
            "javadoc" => Classifier::Javadoc,
            _ if value.starts_with("jdk") => Classifier::Jdk(value),
            _ => Classifier::Unknown(value),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum Packaging {
    Pom,
    Jar,
    War,
    Ear,
    Ejb,
    Rar,
    Par,
    Unknown(String),
}

impl Default for Packaging {
    fn default() -> Self {
        Packaging::Jar
    }
}

impl From<String> for Packaging {
    fn from(value: String) -> Self {
        match value.as_str() {
            "pom" => Packaging::Pom,
            "jar" => Packaging::Jar,
            "war" => Packaging::War,
            "ear" => Packaging::Ear,
            "ejb" => Packaging::Ejb,
            "rar" => Packaging::Rar,
            "par" => Packaging::Par,
            _ => Packaging::Unknown(value),
        }
    }
}

impl From<Packaging> for String {
    fn from(value: Packaging) -> Self {
        match value {
            Packaging::Pom => "pom".into(),
            Packaging::Jar => "jar".into(),
            Packaging::War => "war".into(),
            Packaging::Ear => "ear".into(),
            Packaging::Ejb => "ejb".into(),
            Packaging::Rar => "rar".into(),
            Packaging::Par => "par".into(),
            Packaging::Unknown(s) => s,
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
enum MavenScope {
    Compile,
    // default, available at compile-time and runtime
    Provided,
    // available at compile-time only, but is still required at runtime
    Runtime,
    // only available at runtime
    Test,
    // only available at test-compile-time and test-runtime
    System, // required at compile-time and runtime, but is not included in the project
}

impl Default for MavenScope {
    fn default() -> Self {
        MavenScope::Compile
    }
}

impl From<String> for MavenScope {
    fn from(value: String) -> Self {
        match value.as_str() {
            "compile" => MavenScope::Compile,
            "provided" => MavenScope::Provided,
            "runtime" => MavenScope::Runtime,
            "test" => MavenScope::Test,
            "system" => MavenScope::System,
            _ => MavenScope::default(),
        }
    }
}

/// We dont know if a provided or system scoped dependency is used at runtime
impl From<MavenScope> for Scope {
    fn from(value: MavenScope) -> Self {
        match value {
            MavenScope::Runtime => Scope::Runtime,
            MavenScope::Test => Scope::Test,
            _ => Scope::All,
        }
    }
}

impl Artifact {
    fn pom(group_id: &str, artifact_id: &str, version: &str) -> Self {
        Artifact {
            group_id: Some(group_id.to_owned()),
            artifact_id: Some(artifact_id.to_owned()),
            version: Some(version.to_owned()),
            packaging: Some(Packaging::Pom),
            ..Default::default()
        }
    }

    fn new(
        group_id: &str,
        artifact_id: &str,
        version: &str,
        scope: Option<MavenScope>,
        packaging: Option<Packaging>,
        classifier: Option<Classifier>,
    ) -> Self {
        Artifact {
            group_id: Some(group_id.to_owned()),
            artifact_id: Some(artifact_id.to_owned()),
            version: Some(version.to_owned()),
            scope,
            packaging,
            classifier,
        }
    }

    fn interpolate_version(&mut self, properties: &HashMap<String, String>) -> Self {
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

    fn with_packaging(&self, packaging: Packaging) -> Self {
        Artifact {
            packaging: Some(packaging),
            ..self.clone()
        }
    }

    fn same_ga(&self, other: &Self) -> bool {
        self.group_id == other.group_id && self.artifact_id == other.artifact_id
    }

    fn normalize(self, parent: &Self, default_packaging: Packaging) -> Self {
        Artifact {
            group_id: self.group_id.or_else(|| parent.group_id.clone()),
            artifact_id: self.artifact_id.or_else(|| parent.artifact_id.clone()),
            version: self.version.or_else(|| parent.version.clone()),
            packaging: self
                .packaging
                .or_else(|| Some(default_packaging)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone)]
struct Parent {
    artifact: Artifact,
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

fn parse_artifact(n: &Node) -> Artifact {
    Artifact {
        group_id: node_text(n, "groupId"),
        artifact_id: node_text(n, "artifactId"),
        version: node_text(n, "version"),
        scope: node_text(n, "scope").map(|scope| scope.into()),
        packaging: node_text(n, "packaging").map(|pkg| pkg.into()), // TODO dirty hack
        classifier: node_text(n, "classifier").map(|cls| cls.into()),
    }
}

fn parse_parent(n: &Node) -> Option<Parent> {
    let n = node(n, "parent")?;
    Some(Parent {
        artifact: parse_artifact(&n),
    })
}

fn parse_dependencies(n: &Node) -> Vec<Artifact> {
    match node(n, "dependencies") {
        Some(n) => n
            .children()
            .filter(|child| child.is_element() && child.has_tag_name("dependency"))
            .map(|child| parse_artifact(&child))
            .collect(),
        _ => vec![],
    }
}

fn parse_dependency_management(n: &Node) -> Vec<Artifact> {
    match node(n, "dependencyManagement") {
        Some(dm) => parse_dependencies(&dm),
        _ => vec![]
    }
}

fn parse_properties(n: &Node) -> HashMap<String, String> {
    match node(n, "properties") {
        Some(n) => n
            .children()
            .filter(|child| child.is_element())
            .map(|child| (child.tag_name().name().to_owned(), child.text().unwrap().to_owned()))
            .collect(),
        _ => HashMap::new(),
    }
}

// RESOLVER

trait UrlFetcher {
    fn fetch(&self, url: &str) -> Result<String>;
}

struct DefaultUrlFetcher {}

impl UrlFetcher for DefaultUrlFetcher {
    fn fetch(&self, url: &str) -> Result<String> {
        task::block_on(async {
            let mut response = surf::get(url).await.map_err(|e| anyhow::anyhow!(e))?;
            response.body_string().await.map_err(|e| anyhow::anyhow!(e))
        })
    }
}

struct Repository {
    pub base_url: String,
}

struct Resolver {
    pub repository: Repository,
    pub project_cache: HashMap<Artifact, Project>,
}

impl Default for Resolver {
    fn default() -> Self {
        Resolver {
            repository: Repository {
                base_url: "https://repo.maven.apache.org/maven2".into(),
            },
            project_cache: HashMap::new(),
        }
    }
}

impl Resolver {
    fn create_url(&self, id: &Artifact) -> Result<String> {
        // a little helper
        fn require<'a, F, D>(id: &'a Artifact, f: F, field_name: &D) -> Result<&'a String>
            where
                F: Fn(&Artifact) -> Option<&String>,
                D: std::fmt::Debug,
        {
            Ok(f(id).context(format!("missing parameter {:?} {:?}", id, field_name))?)
        }

        let group_id = require(id, |id| id.group_id.as_ref(), &"groupId")?;
        let artifact_id = require(id, |id| id.artifact_id.as_ref(), &"artifactId")?;
        let version = require(id, |id| id.version.as_ref(), &"version")?;

        let packaging = &id.packaging.clone().unwrap_or_default();
        //let packaging = require(id, |id| id.packaging.map(|p|String::from(p)).unwrap_or_default().as_ref(), &"packaging")?;

        let mut url = format!(
            "{}/{}/{}/{}/{}-{}",
            self.repository.base_url,
            group_id.replace(".", "/"),
            artifact_id,
            version,
            artifact_id,
            version
        );

        if let Some(classifier) = &id.classifier {
            url += &format!("-{}", classifier);
        }

        let package = String::from(packaging.clone());
        url += &format!(".{}", package);

        Ok(url)
    }
}
/*
fn normalize_gavs(
    dependencies: HashMap<DependencyKey, MavenDependency >,
    parent: &Artifact,
    default_packaging: &str,
) -> HashMap<DependencyKey, MavenDependency > {
    dependencies
        .into_iter()
        .map(|(_, dep)| {
            let dep = dep.normalize(parent, default_packaging);
            (dep.get_key(), dep)
        })
        .collect()
}

impl Resolver {
    fn create_url(&self, id: &Artifact) -> Result<String> {
        // a little helper
        fn require<'a, F, D>(id: &'a Artifact, f: F, field_name: &D) -> Result<&'a String>
        where
            F: Fn(&Artifact) -> Option<&String>,
            D: std::fmt::Debug,
        {
            Ok(f(id).context(format!("missing parameter {:?} {:?}", id, field_name))?)
        }

        let group_id = require(id, |id| id.group_id.as_ref(), &"groupId")?;
        let artifact_id = require(id, |id| id.artifact_id.as_ref(), &"artifactId")?;
        let version = require(id, |id| id.version.as_ref(), &"version")?;
        let packaging = require(id, |id| id.packaging.as_ref(), &"packaging")?;

        let mut url = format!(
            "{}/{}/{}/{}/{}-{}",
            self.repository.base_url,
            group_id.replace(".", "/"),
            artifact_id,
            version,
            artifact_id,
            version
        );

        if let Some(classifier) = &id.classifier {
            url += &format!("-{}", classifier);
        }

        url += &format!(".{}", packaging);

        Ok(url)
    }

    fn build_effective_pom<UF, P>(
        &mut self,
        project_id: &Artifact,
        url_fetcher: &UF,
        pom_parser: &P,
    ) -> Result<Project>
    where
        UF: UrlFetcher,
        P: DepGraphParser,
    {
        println!("building an effective pom for {:?}", project_id);

        let project_id = &project_id.with_packaging("pom");

        let mut project = self.fetch_project(project_id, url_fetcher, pom_parser)?;

        if let Some(version) = &project_id.version {
            project
                .properties
                .insert("project.version".to_owned(), version.clone());
        }

        // merge in the dependencies from the parent POM
        if let Some(parent) = &project.parent {
            let parent_project =
                self.build_effective_pom(&parent.artifact, url_fetcher, pom_parser)?;

            println!("got a parent POM: {:?}", parent_project.artifact);

            let extra_deps = parent_project
                .dependencies
                .into_iter()
                .filter(|(dep_key, _)| !project.dependencies.contains_key(dep_key))
                .collect::<HashMap<_, _>>();

            project.dependencies.extend(extra_deps);
        }

        if let Some(mut project_dm) = project.dependency_management.clone() {
            for (_, dep) in &mut project_dm.dependencies {
                dep.artifact = dep.artifact.interpolate(&project.properties);
            }

            let boms: Vec<MavenDependency > = project_dm
                .dependencies
                .iter()
                .filter(|(_, dep)| dep.scope.as_deref() == Some("import"))
                .map(|(_, dep)| dep.clone())
                .collect();

            for bom in boms {
                println!("got a BOM artifact: {:?}", bom.artifact);

                // TODO add protection against infinite recursion
                let bom_project =
                    self.build_effective_pom(&bom.artifact, url_fetcher, pom_parser)?;

                if let Some(DependencyManagement {
                    dependencies: bom_deps,
                }) = bom_project.dependency_management
                {
                    project_dm.dependencies.extend(bom_deps);
                }
            }
        };

        Ok(project)
    }

    fn fetch_project<UF, P>(
        &mut self,
        project_id: &Artifact,
        url_fetcher: &UF,
        pom_parser: &P,
    ) -> Result<Project>
    where
        UF: UrlFetcher,
        P: DepGraphParser,
    {
        // we're looking only for POMs here
        let project_id = project_id.with_packaging("pom");

        // check the cache first
        if let Some(cached_project) = self.project_cache.get(&project_id) {
            println!("returning from cache {:?}...", project_id);
            return Ok(cached_project.clone());
        }

        // grab the remote POM
        let url = self.create_url(&project_id)?;

        println!("fetching {:?}...", url);

        let text = url_fetcher.fetch(&url)?;

        let mut project = pom_parser.parse(text)?;
        // make sure the packaging type is set to "pom"
        let mut project_id = project.artifact.with_packaging("pom");

        // TODO consider moving this to build_effective_pom
        // update the parent and fill-in the project's missing properties using the parent's GAV
        if let Some(parent) = &project.parent {
            let parent = parent.artifact.with_packaging("pom");

            project_id = project_id.normalize(&parent, "pom");

            // normalize dependency GAVs
            project.dependencies = normalize_gavs(project.dependencies, &parent, "jar");
            project.dependency_management = project.dependency_management.map(|mut dm| {
                dm.dependencies = normalize_gavs(dm.dependencies, &parent, "jar");
                dm
            });

            // save the updated FQN
            project.parent = project.parent.map(|mut p| {
                p.artifact = parent;
                p
            });
        }

        // save the updated FQN
        project.artifact = project_id.clone();

        // we're going to save all parsed projects into a HashMap
        // as a "cache"
        println!("caching {:?}", project_id);
        self.project_cache.insert(project_id, project.clone());
        Ok(project)
    }
}
*/

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependencies() {
        let artifact = Artifact::pom("org.jetbrains.kotlin", "kotlin-stdlib", "1.9.22");
        // let artifact = Artifact::pom("io.ktor", "ktor-server-core", "2.3.7");
        let parser = MavenParser {};
        let fetcher = DefaultUrlFetcher {};

        let resolver = Resolver::default();
        let url = resolver.create_url(&artifact).unwrap();
        let text = fetcher.fetch(&url).unwrap();

        let graph = parser.parse(text).unwrap();

        println!("{}", graph);
    }

    #[test]
    fn interpolation() {
        let artifact = Artifact::pom("org.jetbrains.kotlin", "kotlin-stdlib", "1.9.22");
        let fetcher = DefaultUrlFetcher {};
        let resolver = Resolver::default();
        let url = resolver.create_url(&artifact).unwrap();
        let text = fetcher.fetch(&url).unwrap();

        let doc = roxmltree::Document::parse(&text).unwrap();
        let root_node = doc.root();
        let project_node = node(&root_node, "project").unwrap();

        let project = Project::new(project_node);

        for dep in project.dependencies {
            println!{"{dep:?}"}
        }
        for dep in project.dependency_management {
            println!{"{dep:?}"}
        }
    }
}

