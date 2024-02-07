use manifest::config::Config;
use util::{buildk_output::BuildkOutput, terminal::{Terminal, Printable}};

use crate::Command;

impl Command {
    pub fn config(
        &mut self, 
        config: &Config,
        _terminal: &mut Terminal,
    ) -> BuildkOutput {
        let output = BuildkOutput::default();

        config.print(_terminal);

        output
    }
}
