use config::config::Config;
use config::dependencies::dependency::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Colorize, ColorRoulette, Colors};
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

            let colors = ColorRoulette::default();
            Self::decend(dependency, colors, 2);
        });

        output.conclude(PartialConclusion::SUCCESS);
        output
    }

    fn decend(dependency: &Dependency, mut colors: ColorRoulette, depth: usize) {
        let color = colors.next_color();
        dependency.transitives().iter().for_each(|transitive| {
            let display = format!("{:>depth$}{:<14}{}", "", "[transitive]", transitive.name, depth=depth);
            println!("{}", display.colorize(&color))
        });
    }
}
