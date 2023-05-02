use std::path::PathBuf;
use std::str::FromStr;

use toml_edit::Document;

use crate::project::project::Project;
use crate::section::Section;

mod output;

#[allow(clippy::module_inception)]
pub mod project;

#[cfg(test)]
mod test;

fn current_dir() -> PathBuf {
    std::env::current_dir().expect("could not find the current directory")
}

pub(crate) fn project(data: &Document) -> Option<Project> {
    let projects = data.as_table().into_iter().filter_map(|(key, value)| {
        match Section::from_str(key) {
            Ok(Section::Project) => {
                match value.as_table() {
                    None => None,
                    Some(table) => {
                        let main = match table.get("main") {
                            Some(item) => item.as_str(),
                            None => None,
                        };

                        let path = match table.get("path") {
                            Some(item) => item.as_str(),
                            None => None
                        };

                        let relative_path = match table.get("relative-path") {
                            Some(item) => item.as_str(),
                            None => None,
                        };

                        match Project::new(main, path, relative_path) {
                            Ok(project) => Some(project),
                            Err(e) => {
                                eprintln!("Will configure default project settings due to:\n{e}");
                                Some(Project::default())
                            }
                        }
                    }
                }
            }
            _ => None,
        }
    }).collect::<Vec<Project>>();
    projects.into_iter().next()
}
