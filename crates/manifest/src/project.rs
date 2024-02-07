use crate::Section;
use anyhow::{ensure, Result};
use util::terminal::Printable;
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::Document;

#[derive(Clone)]
pub struct Project {
    pub path: PathBuf,
    pub src: PathBuf,
    pub test: PathBuf,
    pub out: ProjectOutput,
    main: String,
}

impl Default for Project {
    fn default() -> Self {
        let path = current_dir();

        Project {
            main: String::from("Main.kt"),
            src: path.join("src"),
            test: path.join("test"),
            out: ProjectOutput::new(path.join("out")),
            path,
        }
    }
}

impl Project {
    pub fn new(
        main: Option<&str>,
        src: Option<&str>,
        test: Option<&str>,
    ) -> Result<Self> {
        let src = match src {
            None => current_dir().join(PathBuf::from("src")),
            Some(src) => current_dir().join(PathBuf::from(src)) 
        };
        //ensure!(src.is_dir(), format!("{:?} must be a directory.", src));
        ensure!(src.is_absolute(), "configured project.src must be an absolute path");

        let test = match test {
            None => current_dir().join(PathBuf::from("test")),
            Some(test) => current_dir().join(PathBuf::from(test)) 
        };
        //ensure!(test.is_dir(), format!("{:?} must be a directory.", test));
        ensure!(test.is_absolute(), "configured project.test must be an absolute path");

        Ok(Self {
            path: current_dir(),
            src,
            test,
            out: ProjectOutput::new(current_dir().join("out")),
            main: main.unwrap_or("Main.kt").to_string(),
        })
    }
    pub fn compiled_main_file(&self) -> String {
        self.main.replace(".kt", "Kt")
    }
}

impl Printable for Project {
    fn print(&self, terminal: &mut util::terminal::Terminal) {
        terminal.print(&format!("{:<26}{}", "project.path", self.path.display()));
        terminal.print(&format!("{:<26}{}", "project.src", self.src.display()));
        terminal.print(&format!("{:<26}{}", "project.test", self.test.display()));
        terminal.print(&format!("{:<26}{}", "project.main", self.main));
        self.out.print(terminal);
    }
}

#[derive(Clone)]
pub struct ProjectOutput {
    pub path: PathBuf,
    pub src: PathBuf,
    pub cache: PathBuf,
    pub test: PathBuf,
    pub test_report: PathBuf,
    pub jar: PathBuf,
}

impl ProjectOutput {
    pub(crate) fn new(path: PathBuf) -> Self {
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

impl Printable for ProjectOutput {
    fn print(&self, terminal: &mut util::terminal::Terminal) {
        terminal.print(&format!("{:<26}{}", "project.out.path", self.path.display()));
        terminal.print(&format!("{:<26}{}", "project.out.cache", self.cache.display()));
        terminal.print(&format!("{:<26}{}", "project.out.src", self.src.display()));
        terminal.print(&format!("{:<26}{}", "project.out.test", self.test.display()));
        terminal.print(&format!("{:<26}{}", "project.out.test-report", self.test_report.display()));
        terminal.print(&format!("{:<26}{}", "project.out.jar", self.jar.display()));
    }
}

fn current_dir() -> PathBuf {
    std::env::current_dir().expect("current path not found")
}

pub(crate) fn project(data: &Document) -> Option<Project> {
    let projects = data
        .as_table()
        .into_iter()
        .filter_map(|(key, value)| match Section::from_str(key) {
            Ok(Section::Project) => match value.as_table() {
                None => None,
                Some(table) => {
                    let main = match table.get("main") {
                        Some(item) => item.as_str(),
                        None => None,
                    };

                    let src = match table.get("src") {
                        Some(item) => item.as_str(),
                        None => None,
                    };

                    let test = match table.get("test") {
                        Some(item) => item.as_str(),
                        None => None,
                    };

                    match Project::new(main, src, test) {
                        Ok(project) => Some(project),
                        Err(e) => {
                            eprintln!("Will configure default project settings due to:\n{e}");
                            Some(Project::default())
                        }
                    }
                }
            },
            _ => None,
        })
        .collect::<Vec<Project>>();
    projects.into_iter().next()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::project::project;

    #[test]
    fn main() {
        let project = project(
            &r#"
[project]
main = "incredible.kt"
"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(project.compiled_main_file(), "incredibleKt")
    }

    #[test]
    fn default_path() {
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(
            project.path,
            PathBuf::from(std::env::current_dir().unwrap())
        )
    }

    #[test]
    fn path() {
        let project = project(
            &r#"
[project]
path = "/Users"
"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(project.path, PathBuf::from("/Users"))
    }

    #[test]
    fn relative_path() {
        let current_path: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(
            &r#"
[project]
relative-path = "src"
"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(project.path, current_path.join("src"))
    }

    #[test]
    fn default_src() {
        let current_path: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(project.src, current_path.join("src"))
    }

    #[test]
    fn default_test() {
        let current_path: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(project.test, current_path.join("test"))
    }

    #[test]
    fn default_out_path() {
        let default_out: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(project.out.path, default_out.join("out"));
    }

    #[test]
    fn default_out_src() {
        let default_out: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(project.out.src, default_out.join("out").join("src"));
    }

    #[test]
    fn default_out_test() {
        let default_out: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(project.out.test, default_out.join("out").join("test"));
    }

    #[test]
    fn default_out_test_report() {
        let default_out: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(
            project.out.test_report,
            default_out.join("out").join("test-report")
        );
    }

    #[test]
    fn default_out_cache() {
        let default_out: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(
            project.out.cache,
            default_out.join("out").join("cache.json")
        );
    }

    #[test]
    fn default_out_jar() {
        let default_out: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(&r#"[project]"#.parse().unwrap()).unwrap();
        assert_eq!(project.out.jar, default_out.join("out").join("app.jar"));
    }
}
