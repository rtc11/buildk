use util::buildk_output::BuildkOutput;

use crate::Command;


pub (crate) struct Config<'a> {
    buildk: &'a manifest::config::BuildK,
}

impl <'a> Command for Config<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let output = BuildkOutput::new("config");

        println!("\r{}", self.buildk);

        output
    }
}

impl <'a> Config<'_> {
    pub fn new(buildk: &'a manifest::config::BuildK) -> Config<'a> {
        Config { buildk }
    }
}
