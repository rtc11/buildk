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
