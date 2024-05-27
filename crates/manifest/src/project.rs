use crate::Section;
use std::fmt::Display;
use std::path::PathBuf;
use std::str::FromStr;
use toml_edit::DocumentMut;

#[derive(Clone)]
pub struct Project {
    pub path: PathBuf,
    pub src: PathBuf,
    pub test: PathBuf,
    pub out: PathBuf,
    pub main: String,
}

impl Project {
    pub fn out_paths(&self) -> ProjectOutput {
        ProjectOutput::new(&self)
    }
}

impl Default for Project {
    fn default() -> Self {
        let path = current_dir();

        Project {
            main: String::from("Main.kt"),
            src: path.join("src"),
            test: path.join("test"),
            out: path.join("out"),
            path,
        }
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project", self.path.display())?;
        writeln!(f, "{:<26}{}", "project.src", self.src.display())?;
        writeln!(f, "{:<26}{}", "project.test", self.test.display())?;
        writeln!(f, "{:<26}{}", "project.main", self.main)?;

        write!(f, "{}", self.out_paths())?;

        write!(f, "")
    }
}

#[derive(Clone)]
pub struct ProjectOutput {
    pub path: PathBuf,
    pub src: PathBuf,
    pub cache: PathBuf,
    pub test: PathBuf,
    pub test_report: PathBuf,
    pub release: PathBuf,
}

impl ProjectOutput {
    pub fn new(project: &Project) -> Self {
        ProjectOutput {
            src: project.out.join("src"),
            cache: project.out.join("cache.json"),
            test: project.out.join("test"),
            test_report: project.out.join("test-report"),
            release: project.out.join("app.jar"),
            path: project.out.clone(),
        }
    }
}

impl Display for ProjectOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project.out", self.path.display())?;
        writeln!(f, "{:<26}{}", "project.out.cache", self.cache.display())?;
        writeln!(f, "{:<26}{}", "project.out.src", self.src.display())?;
        writeln!(f, "{:<26}{}", "project.out.test", self.test.display())?;
        writeln!(f, "{:<26}{}", "project.out.test-report", self.test_report.display())?;
        writeln!(f, "{:<26}{}", "project.out.release", self.release.display())
    }
}

fn current_dir() -> PathBuf {
    std::env::current_dir().expect("path to current working directory")
}

impl From<DocumentMut> for Project {
    fn from(value: DocumentMut) -> Self {
        value
            .as_table()
            .into_iter()
            .filter_map(|(key, value)| {
                if let Ok(Section::Project) = Section::from_str(key) {
                    if let Some(table) = value.as_table() {
                        let path = table.get("path").map_or(current_dir(), |it| {
                            PathBuf::from(it.as_str().expect("path to project"))
                        });

                        let src = table
                            .get("src")
                            .map_or("src", |it| it.as_str().expect("path to src"));

                        let test = table
                            .get("test")
                            .map_or("test", |it| it.as_str().expect("path to test"));

                        let out = table
                            .get("out")
                            .map_or("out", |it| it.as_str().expect("path to out"));

                        let main = table
                            .get("main")
                            .map_or("Main.kt", |it| it.as_str().expect("path to main"));

                        Some(Project {
                            src: path.join(src),
                            test: path.join(test),
                            out: path.join(out),
                            main: main.to_string(),
                            path,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next()
            .unwrap_or_default()
    }
}

/* #[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn set_main() {
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
    fn relative_path() {
        let current_path: PathBuf = std::env::current_dir().unwrap().into();
        let project = project(
            &r#"
[project]
src = "hello"
"#
            .parse()
            .unwrap(),
        )
        .unwrap();

        assert_eq!(project.src, current_path.join("hello"))
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
        assert_eq!(project.out.release, default_out.join("out").join("app.jar"));
    }
} */
