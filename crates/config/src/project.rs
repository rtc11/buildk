use std::env;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

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
        let path = env::current_dir().expect("could not find the current directory");

        Project {
            main: String::from("main.kt"),
            src: path.join("src"),
            test: path.join("test"),
            out: Output::new(path.join("out")),
            path,
        }
    }
}

impl Project {
    pub fn new(main: Option<&str>, path: Option<&str>) -> Self {
        let path = path
            .map(|p| PathBuf::from(p))
            .unwrap_or(env::current_dir().expect("could not find the current directory"));

        Self {
            main: main.unwrap_or("main.kr").to_string(),
            src: path.join("src"),
            test: path.join("test"),
            out: Output::new(path.join("out")),
            path,
        }
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
