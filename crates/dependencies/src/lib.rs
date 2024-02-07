use std::fmt::Display;

use anyhow::Result;
use gryf::{core::{marker::Directed, index::VertexIndex}, Graph};

mod buildk_parser;
mod gradle_parser;
mod maven_parser;

pub trait DepGraphParser {
    fn parse(&self, input: String) -> Result<DepGraph>;
}

#[derive(Default)]
pub struct DepGraph {
    // TODO: When cycel detected, we dont have to include it as its already in the graph
    // TODO: When multiple versions are detected, use the newest one
    graph: Graph<Dependency, (), Directed>,
}

impl DepGraph {
    pub fn add(&mut self, dependency: Dependency) -> VertexIndex {
        self.graph.add_vertex(dependency)
    }

    pub fn connect(&mut self, from: VertexIndex, to: VertexIndex) {
        self.graph.add_edge(from, to, ());
    }
}

impl Display for DepGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, dep) in self.graph.vertices() {
            writeln!(f, "{:?}:{}", idx, dep)?;
        }
        Ok(())
    }

}

#[derive(Clone)]
pub struct Dependency {
    group: String,
    artifact: String,
    version: String,
    scope: Scope, // todo: change to enum when all values are known
}

#[derive(Clone)]
enum Scope {
    /// Default scope, includes all
    All,

    /// Compile time only
    Compile,

    /// Runtime only
    Runtime,

    /// Test only
    Test,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::All
    }
}


impl Display for Scope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Scope::All => write!(f, "all"),
            Scope::Compile => write!(f, "compile"),
            Scope::Runtime => write!(f, "runtime"),
            Scope::Test => write!(f, "test"),
        }
    }
}

impl Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{} scope:{}", self.group, self.artifact, self.version, self.scope)
    }
}
