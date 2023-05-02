use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::ensure;
use toml_edit::Document;

use crate::manifest::Section;

pub struct Project {
    pub path: PathBuf,
    pub src: PathBuf,
    pub test: PathBuf,
    pub out: Output,
    main: String,
}

pub struct Output {
    pub path: PathBuf,
    pub src: PathBuf,
    pub cache: PathBuf,
    pub test: PathBuf,
    pub test_report: PathBuf,
    pub jar: PathBuf,
}

impl Output {
    fn new(path: PathBuf) -> Self {
        Self {
            src: path.join("src"),
            cache: path.join("cache.json"),
            test: path.join("test"),
            test_report: path.join("test-report"),
            jar: path.join("app.jar"),
            path,
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        let path = current_dir();

        Project {
            main: String::from("Main.kt"),
            src: path.join("src"),
            test: path.join("test"),
            out: Output::new(path.join("out")),
            path,
        }
    }
}

impl Project {
    pub fn new(
        main: Option<&str>,
        path: Option<&str>,
        relative_path: Option<&str>,
    ) -> anyhow::Result<Self> {
        let path = path
            .map(PathBuf::from)
            .or(relative_path.map(|relative| current_dir().join(relative)))
            .unwrap_or(current_dir());

        ensure!(path.is_dir(), "project path be a directory. Verify your 'path' under [project] in buildk.toml");
        ensure!(path.is_absolute(), "project path must be an absolute path. Verify your 'path' under [project] in  buildk.toml");

        Ok(Self {
            main: main.unwrap_or("Main.kt").to_string(),
            src: path.join("src"),
            test: path.join("test"),
            out: Output::new(path.join("out")),
            path,
        })
    }
    pub fn compiled_main_file(&self) -> String {
        self.main.replace(".kt", "Kt")
    }
}

fn current_dir() -> PathBuf {
    env::current_dir().expect("could not find the current directory")
}

pub fn project(data: &Document) -> Option<Project> {
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

impl Display for Project {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project.path", self.path.display())?;
        writeln!(f, "{:<26}{}", "project.src", self.src.display())?;
        writeln!(f, "{:<26}{}", "project.test", self.test.display())?;
        writeln!(f, "{:<26}{} ({})", "project.main", self.main, self.compiled_main_file())?;
        write!(f, "{}", self.out)
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project.out.path", self.path.display())?;
        writeln!(f, "{:<26}{}", "project.out.cache", self.cache.display())?;
        writeln!(f, "{:<26}{}", "project.out.src", self.src.display())?;
        writeln!(f, "{:<26}{}", "project.out.test", self.test.display())?;
        writeln!(f, "{:<26}{}", "project.out.test-report", self.test_report.display())?;
        writeln!(f, "{:<26}{}", "project.out.jar", self.jar.display())
    }
}
