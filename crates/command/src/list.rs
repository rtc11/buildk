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
                true => println!("{}", format!("{:<12}{}", "[cached]", dependency.filename.display()).as_gray()),
                false => println!("{}", format!("{:<12}{}", "[fetched]", dependency.filename.display()).as_green()),
            };
        });

        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
