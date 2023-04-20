use anyhow::{bail, Result};
use fs::toml::Config;

mod target;
mod build;
mod clean;
mod run;
mod test;

// TODO: require intsalled on OS for smaller target size?
const KOTLINC: &'static str = "../kotlinc/bin/kotlinc-jvm";

pub enum Cmd {
    Clean,
    Build,
    Test,
    Run,
    Release,
}

impl TryFrom<String> for Cmd {
    type Error = anyhow::Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        match value.as_str() {
            "clean" => Ok(Cmd::Clean),
            "build" => Ok(Cmd::Build),
            "test" => Ok(Cmd::Test),
            "run" => Ok(Cmd::Run),
            "release" => Ok(Cmd::Release),
            _ => bail!("Invalid command {}", value)
        }
    }
}

pub fn build(config: &Config) -> Vec<Result<String>> {
    vec![
        build::src(config),
        build::test(config),
    ]
}

pub fn clean(config: &Config) -> Vec<Result<String>> {
    vec![
        clean::build_dir(config),
    ]
}

pub fn target(config: &Config) -> Vec<Result<String>> {
    vec![
        build::src(config),
        target::jar(config),
    ]
}

pub fn run(config: &Config) -> Vec<Result<String>> {
    vec![
        run::app(config),
    ]
}

pub fn test(config: &Config) -> Vec<Result<String>> {
    vec![
        build::src(config),
        build::test(config),
        test::run(config),
    ]
}
