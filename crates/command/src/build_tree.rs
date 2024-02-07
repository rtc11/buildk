use crate::Command;
use manifest::config::Config;
use gryf::Graph;
use gryf::algo::TopoSort;
use util::terminal::{Terminal, Printable};
use std::path::{PathBuf, Path};
use util::buildk_output::BuildkOutput;
use util::paths::all_files_recursive;
use util::{PartialConclusion, StringExtras};

impl Command {
    pub fn build_tree(
        &self, 
        config: &Config,
        _terminal: &mut Terminal,
    ) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        match sort_by_imports(config){
            Ok(sorted) => {
                //let project_path = &config.manifest.project.path;
                //sorted.iter().for_each(|file| println!("\r{:?}", file.strip_prefix(project_path).unwrap()));
                output.stdout(format!("{sorted:?}"));
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

pub fn sort_by_imports(config: &Config) -> anyhow::Result<Vec<PathBuf>> {
    let mut graph = Graph::new_directed();

    let paths = all_files_recursive(vec![], config.manifest.project.src.clone())?;
    paths.iter()
        .filter(|path| path.extension().unwrap_or_default() == "kt")
        .map(Path::new)
        .map(HeaderKt::parse)
        .filter_map(|header| header.ok())
        .for_each(|header| {
            graph.add_vertex(header);
        });

    graph.connect_vertices(|u, v| v.has_dependency(u).then_some(()));

    let sorted = TopoSort::on(&graph)
        .run()
        .map(|r| r.map(|v| graph[v].file.clone()))
        .collect::<Result<Vec<_>, _>>();

    Ok(sorted?)
}

#[derive(Clone, Default, Debug)]
pub struct HeaderKt {
    pub file: PathBuf,
    pub package: String,
    imports: Vec<String>,
}

impl HeaderKt {
    pub fn parse(file: &Path) -> anyhow::Result<HeaderKt> {
        let content = util::paths::read(file)?;
        let file = file.to_path_buf();
        let mut package = String::new();
        let mut imports = Vec::new();

        for line in content.lines() {
            match line {
                line if line.starts_with("package ") => package = line.replace("package ", "") .replace_after_last("."),
                line if line.starts_with("import ") => imports.push( line.replace("import ", "").replace_after_last(".")),
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

impl Printable for HeaderKt {
    fn print(&self, terminal: &mut Terminal) {
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

        terminal.print(&s);
    }
}

