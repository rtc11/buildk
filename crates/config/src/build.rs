use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Build {
    #[serde(default = "Build::default_output")]
    pub output: PathBuf,
    #[serde(default = "Build::default_src")]
    pub src: PathBuf,
    #[serde(default = "Build::default_test")]
    pub test: PathBuf,
    #[serde(default = "Build::default_cache")]
    pub cache: PathBuf,
    #[serde(default = "Build::default_target")]
    pub target_jar: PathBuf,
}

impl Build {
    fn default_output() -> PathBuf { PathBuf::from("build") }
    fn default_src() -> PathBuf { PathBuf::from("src") }
    fn default_test() -> PathBuf { PathBuf::from("test") }
    fn default_target() -> PathBuf { PathBuf::from("app.jar") }
    fn default_cache() -> PathBuf { PathBuf::from("cache.json") }

    pub fn output_src(&self) -> PathBuf { self.output.join(&self.src) }
    pub fn output_test(&self) -> PathBuf { self.output.join(&self.test) }
    pub fn output_test_report(&self) -> PathBuf { self.output_test().join("report")}
    pub fn output_target(&self) -> PathBuf { self.output.join(&self.target_jar) }
    pub fn output_cache(&self) -> PathBuf { self.output.join(&self.cache) }
}

impl Display for Build {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:<26}{}", "build.src", self.src.display())?;
        writeln!(f, "{:<26}{}", "build.test", self.test.display())?;
        writeln!(f, "{:<26}{}", "build.output", self.output.display())?;
        writeln!(f, "{:<26}{}", "build.output.src", self.output_src().display())?;
        writeln!(f, "{:<26}{}", "build.output.test", self.output_test().display())?;
        writeln!(f, "{:<26}{}", "build.output.target", self.output_target().display())?;
        writeln!(f, "{:<26}{}", "build.output.cache", self.output_cache().display())
    }
}