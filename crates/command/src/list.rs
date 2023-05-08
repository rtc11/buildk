use config::config::Config;
use config::dependencies::dependency::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::Command;

impl Command {
    pub fn list(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        list_dependencies(&config.manifest.dependencies, 0);

        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}

fn list_dependencies(dependencies: &[Dependency], depth: usize) {
    let color = Color::get_index(depth);
    dependencies.iter().for_each(|dep| {
        let status = match dep.is_cached() {
            true => "[cached]",
            false => "[missing]",
        };
        let display = format!(
            "{:>depth$}{:<14}{}:{}",
            "",
            status,
            dep.name,
            dep.version,
            depth=depth*2,
        );
        println!("{}", display.colorize(&color));
        list_dependencies(&dep.transitives(), depth + 1)
    })
}
