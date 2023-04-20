use std::fs;

use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub project: Project,
    #[serde(default)]
    build: Build,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Project {
    main: String,
    #[serde(default = "default_project_dir")]
    pub dir: String,
}

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Build {
    #[serde(default = "default_build_output")]
    output: String,
    #[serde(default = "default_build_src")]
    src: String,
    #[serde(default = "default_build_test")]
    test: String,
}

fn default_build_output() -> String { "/build".to_string() }
fn default_build_src() -> String { "/src".to_string() }
fn default_build_test() -> String { "/test".to_string() }
fn default_project_dir() -> String { ".".to_string() }

impl Config {
    pub fn is_app(&self) -> bool { true }

    pub fn output_jar(&self) -> String {
        format!("{}/{}/app.jar", self.project.dir, self.build.output)
    }

    pub fn output_dir(&self) -> String {
        format!("{}/{}", self.project.dir, self.build.output)
    }

    pub fn src_dir(&self) -> String {
        format!("{}/{}", self.project.dir, self.build.src)
    }

    pub fn test_dir(&self) -> String {
        format!("{}/{}", self.project.dir, self.build.test)
    }

    pub fn output_src(&self) -> String {
        format!("{}/{}/{}", self.project.dir, self.build.output, self.build.src)
    }

    pub fn output_test(&self) -> String {
        format!("{}/{}/{}", self.project.dir, self.build.output, self.build.test)
    }
}

impl Project {
    pub fn main(&self) -> String {
        self.main.replace(".kt", "Kt")
    }
}

pub fn read() -> Config {
    let contents = match fs::read_to_string("test/config.toml") {
        Ok(contents) => contents,
        Err(_) => panic!("config.toml not found."),
    };

    match toml::from_str(&contents) {
        Ok(config) => config,
        Err(e) => panic!("Unable to parse config.toml into: {e}"),
    }
}
