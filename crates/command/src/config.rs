use manifest::config::Config;
use util::buildk_output::BuildkOutput;

use crate::{ConfigCmd, Commands};

impl ConfigCmd for Commands {
    fn config(
        &mut self, 
        config: &Config,
    ) -> BuildkOutput {
        let output = BuildkOutput::new("config");

        println!("\r{}", config);

        output
    }
}
