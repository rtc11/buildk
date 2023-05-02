use std::fmt::{Display, Formatter};
use std::path::PathBuf;

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

impl Display for ProjectOutput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "project.out.path", self.path.display())?;
        writeln!(f, "{:<26}{}", "project.out.cache", self.cache.display())?;
        writeln!(f, "{:<26}{}", "project.out.src", self.src.display())?;
        writeln!(f, "{:<26}{}", "project.out.test", self.test.display())?;
        writeln!(f, "{:<26}{}", "project.out.test-report", self.test_report.display())?;
        writeln!(f, "{:<26}{}", "project.out.jar", self.jar.display())
    }
}
