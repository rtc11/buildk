use util::buildk_output::BuildkOutput;

use crate::Command;


pub (crate) struct Config<'a> {
    config: &'a manifest::config::Config,
}

impl <'a> Command for Config<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let output = BuildkOutput::new("config");

        println!("\r{}", self.config);

        output
    }
}

impl <'a> Config<'_> {
    pub fn new(config: &'a manifest::config::Config) -> Config<'a> {
        Config { config }
    }
}
