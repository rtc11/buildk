use anyhow::Result;

mod release;
mod build;
mod clean;
mod run;
mod test;

const KOTLINC: &'static str = "../kotlinc/bin/kotlinc-jvm";
const PROJECT: &'static str = "test";

// TODO: deduce from buildk.toml file
pub enum Target {
    App,
    Lib,
}

pub fn build() -> Result<String> {
    let src = build::src()?;
    let test = build::test()?;

    Ok(format!("{src}{test}"))
}

pub fn clean() -> Result<String> {
    clean::build_dir()
}

pub fn release(target: Target) -> Result<String> {
    match target {
        Target::App => release::app(),
        Target::Lib => release::lib(),
    }
}

pub fn run() -> Result<String> {
    run::app("MainKt")
}

pub fn test() -> Result<String> {
    test::run()
}

