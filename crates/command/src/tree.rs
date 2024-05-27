use std::fmt::Display;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use gryf::algo::TopoSort;
use gryf::Graph;

use manifest::config::BuildK;
use util::{PartialConclusion, StringExtras};
use util::buildk_output::BuildkOutput;
use util::paths::all_files_recursive;

use crate::Command;

pub(crate) struct Tree {
    files: Vec<PathBuf>,
}

impl Command for Tree {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("tree");

        match self.get_sorted_tree() {
            Ok(files) => {
                self.files = files;
                output.stdout(format!("{self}"));
                output.conclude(PartialConclusion::SUCCESS);
            }
            Err(e) => {
                output.stdout("cyclic dependency detected".to_owned());
                output.stderr(e.to_string());
                output.conclude(PartialConclusion::FAILED);
            }
        }

        output
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for file in &self.files {
            if let Some(filename) = file.file_name() {
                s.push_str(&format!("{}\n", filename.to_string_lossy()));
            }
        }
        write!(f, "{}", s)
    }
}

impl<'a> Tree {
    pub fn new(buildk: &'a BuildK) -> Result<Tree> {
        let manifest = buildk.clone().manifest.context("'tree' is missing manifest.")?;
        let src = manifest.project.src.clone();
        let files = all_files_recursive(vec![], src).unwrap_or_default();
        Ok(Tree { files })
    }

    pub fn get_sorted_tree(&self) -> Result<Vec<PathBuf>> {
        let mut graph = Graph::new_directed();

        self.files
            .iter()
            .filter(|path| path.extension().unwrap_or_default() == "kt")
            .map(Path::new)
            .map(HeaderKt::parse)
            .filter_map(Result::ok)
            .for_each(|header| {
                graph.add_vertex(header);
            });

        graph.connect_vertices(|u, v|
            v.has_dependency(u).then_some(())
        );

        let sorted = TopoSort::on(&graph)
            .run()
            .map(|res| res.map(|v| graph[v].file.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(sorted)
    }
}

#[derive(Clone, Default, Debug)]
pub struct HeaderKt {
    pub file: PathBuf,
    pub package: String,
    imports: Vec<String>,
}

impl HeaderKt {
    // TODO: build a lexer?
    pub fn parse(file: &Path) -> Result<HeaderKt> {
        let content = util::paths::read(file)?;
        let file = file.to_path_buf();
        let mut package = String::new();
        let mut imports = Vec::new();

        for line in content.lines() {
            match line {
                line if line.starts_with("package ") => {
                    package = line.replace("package ", "").replace_after_last(".")
                }
                line if line.starts_with("import ") => {
                    imports.push(line.replace("import ", "").replace_after_last("."))
                }
                "" => {}
                _ => break, // skip rest of file
            }
        }

        Ok(HeaderKt { file, package, imports })
    }

    pub fn has_dependency(&self, other: &HeaderKt) -> bool {
        self.imports.contains(&other.package)
    }
}

