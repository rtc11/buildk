use crate::{Commands, TreeCmd};
use anyhow::Result;
use gryf::algo::TopoSort;
use gryf::Graph;
use manifest::config::Config;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use util::buildk_output::BuildkOutput;
use util::paths::all_files_recursive;
use util::{PartialConclusion, StringExtras};

impl TreeCmd for Commands {
    fn tree(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::new("tree");
        let mut tree = Tree::new(config);

        match tree.sort_by_imports() {
            Ok(_) => {
                output.stdout(format!("{tree}"));
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

pub struct Tree {
    pub files: Vec<PathBuf>,
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

impl Tree {
    pub fn new(config: &Config) -> Self {
        let src = config.manifest.project.src.clone();
        let files = all_files_recursive(vec![], src).unwrap_or_default();
        Tree { files }
    }

    pub fn sort_by_imports(&mut self) -> Result<()> {
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

        self.files = sorted;

        Ok(())
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
                line if line.is_empty() => {}
                _ => break, // skip rest of file
            }
        }

        Ok(HeaderKt {
            file,
            package,
            imports,
        })
    }

    pub fn has_dependency(&self, other: &HeaderKt) -> bool {
        self.imports.contains(&other.package)
    }
}
/*
impl Display for HeaderKt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();

        if self.package.is_empty() {
            let str = format!("{:<20} | ", &self.file.display());
            s.push_str(&str);
        } else {
            let str = format!("{:<20} | ", &self.package);
            s.push_str(&str);
        }

        for import in &self.imports {
            s.push_str(import);
            s.push_str(", ");
        }

        write!(f, "{}", s)
    }
}
*/
