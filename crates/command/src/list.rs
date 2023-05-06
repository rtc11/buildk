use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::PartialConclusion;

use crate::Command;

impl Command {
    pub fn list(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        config.manifest.dependencies.iter().for_each(|dependency| {
            match dependency.is_cached() {
                true => println!("{}", format!("{:<14}{}", "[cached]", dependency.name).as_yellow()),
                false => println!("{}", format!("{:<14}{}", "[fetched]", dependency.name).as_green()),
            };
            dependency.transitives().iter().for_each(|transitive| {
                println!("{}", format!("{:<14}{}", "[transitive]", transitive.name).as_gray())
            });
        });

        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
