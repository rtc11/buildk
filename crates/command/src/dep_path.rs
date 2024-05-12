use manifest::{config::Config, dependencies::Name, manifest::Manifest};
use util::buildk_output::BuildkOutput;

use crate::Command;

pub(crate) struct DepPath<'a> {
    config: &'a Config,
}

impl<'a> Command for DepPath<'a> {
    type Item = String;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let output = BuildkOutput::new("config");

        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest)
            .expect("no buildk.toml found.");

        let arg = Name::from(arg.expect("no arg found."));

        let dep = &manifest
            .dependencies
            .into_iter()
            .find(|dep| dep.name == arg)
            .expect("no dep found.");

        println!("\r{}{}", dep.target_dir.display(), dep.jar);
        output
    }
}
        
impl <'a> DepPath<'_> {
    pub fn new(config: &'a Config) -> DepPath<'a> {
        DepPath { config }
    }
}
