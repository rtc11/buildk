use std::path::PathBuf;

pub mod process_builder;
pub mod process_error;
pub mod hasher;
pub mod paths;
pub mod buildk_output;
pub mod colorize;
pub mod timer;

pub type BuildkResult<T> = anyhow::Result<T>;

pub enum Conclusion {
    SUCCESS,
    FAILED,
}

#[derive(Clone, PartialEq)]
pub enum PartialConclusion {
    INIT,
    CACHED,
    SUCCESS,
    FAILED,
}

pub fn get_kotlinc() -> PathBuf {
    get_kotlin_home().join("bin/kotlinc")
}

pub fn get_kotlin_home() -> PathBuf {
    match option_env!("KOTLIN_HOME") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from("/Users/robin/kotlin/kotlin-v1.8.21/kotlinc"),
    }
}
