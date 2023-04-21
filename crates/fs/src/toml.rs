use std::fmt::{Display, Formatter};
use std::fs;

use serde_derive::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub project: Project,
    #[serde(default)]
    pub build: Build,
}

#[derive(Deserialize, Clone)]
pub struct Project {
    main: String,
    #[serde(default = "default_project_dir")]
    pub dir: String,
}

#[derive(Deserialize, Clone)]
pub struct Build {
    #[serde(default = "default_build_output")]
    pub output: String,
    #[serde(default = "default_build_src")]
    pub src: String,
    #[serde(default = "default_build_test")]
    pub test: String,
}

impl Build {
    pub fn output_src(&self) -> String { format!("{}/{}", self.output, self.src) }
    pub fn output_test(&self) -> String { format!("{}/{}", self.output, self.test) }
    pub fn target(&self) -> String { format!("{}/app.jar", self.output) }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut display = String::new();
        display.push_str(&format!("{:<18}{}\n", "project.main:", self.project.main));
        display.push_str(&format!("{:<18}{}\n", "project.main():", self.project.main_class()));
        display.push_str(&format!("{:<18}{}\n", "project.dir:", self.project.dir));
        display.push_str(&format!("{:<18}{}\n", "build.output:", self.build.output));
        display.push_str(&format!("{:<18}{}\n", "build.src:", self.build.src));
        display.push_str(&format!("{:<18}{}\n", "build.test:", self.build.test));
        display.push_str(&format!("{:<18}{}\n", "is_app():", self.is_app()));
        display.push_str(&format!("{:<18}{}\n", "target():", self.build.target()));
        display.push_str(&format!("{:<18}{}\n", "output_src():", self.build.output_src()));
        display.push_str(&format!("{:<18}{}\n", "output_test():", self.build.output_test()));
        writeln!(f, "{display}")
    }
}

impl Default for Build {
    fn default() -> Self {
        Build {
            output: default_build_output(),
            src: default_build_src(),
            test: default_build_test(),
        }
    }
}

fn default_build_output() -> String { "build".to_string() }

fn default_build_src() -> String { "src".to_string() }

fn default_build_test() -> String { "test".to_string() }

fn default_project_dir() -> String { ".".to_string() }

impl Config {
    pub fn is_app(&self) -> bool { true }
}

impl Project {
    pub fn main_class(&self) -> String { self.main.replace(".kt", "Kt") }
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
