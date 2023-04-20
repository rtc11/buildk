use std::fs;

use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    project_dir: Option<String>,
    build_dir: Option<String>,
    src_dir: Option<String>,
    test_dir: Option<String>,
    app: Option<App>,
    lib: Option<Lib>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct App {
    main: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Lib {
    main: String,
}

impl Config {
    pub fn is_app(&self) -> bool { self.app.is_some() }
    pub fn is_lib(&self) -> bool { self.lib.is_some() }

    pub fn project_dir(&self) -> String {
        self.project_dir.clone().unwrap_or(String::from("."))
    }

    pub fn build_dir(&self) -> String {
        self.build_dir.clone().unwrap_or(format!("{}{}", self.project_dir(), "/build"))
    }

    pub fn src_dir(&self) -> String {
        self.src_dir.clone().unwrap_or(format!("{}{}", self.project_dir(), "/src"))
    }

    pub fn test_dir(&self) -> String {
        self.test_dir.clone().unwrap_or(format!("{}{}", self.project_dir(), "/test"))
    }

    pub fn build_src_dir(&self) -> String {
        format!("{}/{}", self.build_dir(), self.src_dir())
    }

    pub fn build_test_dir(&self) -> String {
        format!("{}/{}", self.build_dir(), self.test_dir())
    }

    pub fn target(&self) -> String {
        match self.is_app() {
            true => format!("{}/app.jar", self.build_dir()),
            false => format!("{}/lib.jar", self.build_dir()),
        }
    }

    pub fn main(&self) -> String {
        let filename = match self.is_app() {
            true => self.app.clone().expect("main not specified").main,
            false => self.lib.clone().expect("main not specified").main,
        };

        filename.replace(".kt", "Kt")
    }
}

pub fn read() -> Config {
    let contents = match fs::read_to_string("test/config.toml") {
        Ok(contents) => contents,
        Err(_) => panic!("config.toml not found."),
    };

    match toml::from_str(&contents) {
        Ok(config) => config,
        Err(_) => panic!("Unable to parse config.toml into"),
    }
}
