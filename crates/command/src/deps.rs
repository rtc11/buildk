use manifest::config::Config;
use manifest::dependencies::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::Command;

const LIST_TRANSITIVE: bool = true;

impl Command {
    pub fn deps(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let stdout = list_dependencies(&config.manifest.dependencies, vec![], 0);
        output.stdout(format!("{stdout}"));
        output.conclude(PartialConclusion::SUCCESS).to_owned()
    }
}

fn list_dependencies<'a>(
    dependencies: &'a [Dependency],
    mut traversed: Vec<&'a str>,
    depth: usize,
) -> String {
    let color = Color::get_index(depth);

    let stdout = dependencies.iter().map(|dep| {
        traversed.push(dep.path.as_str());

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

        let stdout = format!("{}", display.colorize(&color));

        let transitives = &dep
            .transitives()
            .into_iter()
            .filter(|dep| !traversed.contains(&dep.path.as_str()))
            .collect::<Vec<_>>();

        let next = list_dependencies(transitives, traversed.clone(), depth + 1);

        if LIST_TRANSITIVE {
            format!("{}\n{}", stdout, next)
        } else {
            stdout
        }
    }).fold(String::new(), |acc, next| format!("{}{}", acc, next));

    stdout
}
