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

#[derive(Deserialize, Debug, Clone)]
pub struct Build {
    #[serde(default = "default_build_output")]
    output: String,
    #[serde(default = "default_build_src")]
    src: String,
    #[serde(default = "default_build_test")]
    test: String,
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

    pub fn output_jar(&self) -> String {
        let str = format!("{}/{}/app.jar", self.project.dir, self.build.output);
        // println!("output_jar:{str}");
        str
    }

    pub fn output_dir(&self) -> String {
        let str = format!("{}/{}", self.project.dir, self.build.output);
        // println!("output_dir:{str}");
        str
    }

    pub fn src_dir(&self) -> String {
        let str = format!("{}/{}", self.project.dir, self.build.src);
        // println!("src_dir:{str}");
        str
    }

    pub fn test_dir(&self) -> String {
        let str = format!("{}/{}", self.project.dir, self.build.test);
        // println!("test_dir:{str}");
        str
    }

    pub fn output_src(&self) -> String {
        let str = format!("{}/{}/{}", self.project.dir, self.build.output, self.build.src);
        // println!("output_src:{str}");
        str
    }

    pub fn output_test(&self) -> String {
        let str = format!("{}/{}/{}", self.project.dir, self.build.output, self.build.test);
        // println!("output_test:{str}");
        str
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
