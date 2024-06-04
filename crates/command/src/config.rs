use process::{java::Java, kotlin::Kotlin};
use util::buildk_output::BuildkOutput;

use crate::Command;


pub (crate) struct Config<'a> {
    buildk: &'a manifest::config::BuildK,
    kotlin: &'a Kotlin<'a>,
    java: &'a Java<'a>,
}

impl <'a> Command for Config<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let output = BuildkOutput::new("config");

        print!("\r{}", self.kotlin);
        print!("\r{}", self.java);
        println!("\r{}", self.buildk);

        output
    }
}

impl <'a> Config<'_> {
    pub fn new(
        buildk: &'a manifest::config::BuildK,
        kotlin: &'a Kotlin,
        java: &'a Java,
    ) -> Config<'a> {
        Config { buildk, kotlin, java }
    }
}
