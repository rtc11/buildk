use config::config::Config;
use config::dependencies::dependency::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colorize, Colors};
use util::PartialConclusion;

use crate::Command;

impl Command {
    pub fn list(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        config.manifest.dependencies.iter().for_each(|dependency| {
            match dependency.is_cached() {
                true => println!("{}", format!("{:<14}{}", "[cached]", dependency.name).as_gray()),
                false => println!("{}", format!("{:<14}{}", "[fetched]", dependency.name).as_green()),
            };

            Self::decend(dependency, 1);
        });

        output.conclude(PartialConclusion::SUCCESS);
        output
    }

    fn decend(dependency: &Dependency, depth: usize) {
        let color = Color::get_index(depth);
        dependency.transitives().iter().for_each(|transitive| {
            let display = format!(
                "{:>depth$}{:<14}{}:{}",
                "",
                "[transitive]",
                transitive.name,
                transitive.version,
                depth=depth*2,
            );
            println!("{}", display.colorize(&color));
            Self::decend(transitive, depth + 1)
        });
    }
}
