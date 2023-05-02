use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use anyhow::ensure;
use crate::project::current_dir;
use crate::project::output::ProjectOutput;

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
            out: ProjectOutput::new(path.join("out")),
            path,
        })
    }
    pub fn compiled_main_file(&self) -> String {
        self.main.replace(".kt", "Kt")
    }
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
