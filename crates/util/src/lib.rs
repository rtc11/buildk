use std::env;
use std::path::PathBuf;
use crate::process_builder::ProcessBuilder;

pub mod process_builder;
pub mod process_error;
pub mod hasher;
pub mod paths;
pub mod buildk_output;
pub mod cmd;
mod colorize;
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
    PathBuf::from("/Users/robin/kotlin/kotlin-v1.8.21/kotlinc/bin/kotlinc")
}

pub fn get_kotlin_home() -> PathBuf {
    match env::var("KOTLIN_HOME") {
        Ok(kotlin_home) => PathBuf::from(kotlin_home),
        Err(_) => match ProcessBuilder::new("which").arg("kotlinc").output() {
            Ok(output) => {
                match String::from_utf8(output.stdout) {
                    Ok(stdout) => match stdout.strip_suffix("/bin/kotlinc") {
                        Some(kotlin_home) => PathBuf::from(kotlin_home),
                        None => {
                            eprintln!("kotlin home directory not found when calling `which kotlinc`");
                            PathBuf::from("/Users/robin/kotlin/kotlin-v1.8.21/kotlinc")
                            // panic!("kotlin home directory not found when calling `which kotlinc`")
                        },
                    }
                    Err(e) => panic!("Suffix /bin/kotlinc not found: {e}")
                }
            }
            Err(e) => panic!("Failed to find kotlin home directory: {}", e)
        }
    }
}
