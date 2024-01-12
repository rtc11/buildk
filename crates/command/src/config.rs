use manifest::config::Config;
use util::buildk_output::BuildkOutput;

use crate::Command;

impl Command {
    pub fn config(&mut self, config: &Config) -> BuildkOutput {
        let output = BuildkOutput::default();

        println!("\r{config}");

        output
    }
}
