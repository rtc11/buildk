use config::config::Config;
use config::dependencies::dependency::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::Command;

impl Command {
    pub fn deps(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        list_dependencies(&config.manifest.dependencies, vec![], 0);
        println!("deps: {:?}", config.manifest.dependencies.len());

        output
            .conclude(PartialConclusion::SUCCESS)
            .to_owned()
    }
}

fn list_dependencies<'a>(
    dependencies: &'a [Dependency],
    mut traversed: Vec<&'a str>,
    depth: usize,
) {
    let color = Color::get_index(depth);

    dependencies.iter().for_each(|dep| {
        traversed.push(&dep.url.as_str());

        let status = match dep.is_cached() {
            true => "[cached]",
            false => "[missing]",
        };

        let display = format!(
            "\r{:>depth$}{:<14}{}:{}",
            "",
            status,
            dep.name,
            dep.version,
            depth = depth * 2,
        );

        println!("{}", display.colorize(&color));

        let transitives = &dep
            .transitives()
            .into_iter()
            .filter(|dep| !traversed.contains(&dep.url.as_str()))
            .collect::<Vec<_>>();

        list_dependencies(transitives, traversed.clone(), depth + 1)
    })
}
