use dependency::{Package, PackageKind};
use manifest::config::BuildK;
use util::buildk_output::BuildkOutput;

use crate::Command;

#[allow(dead_code)]
pub(crate) struct DepPath<'a> {
    buildk: &'a BuildK,
}

impl<'a> Command for DepPath<'a> {
    type Item = String;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let output = BuildkOutput::new("config");
        let arg = arg.expect("missing arg <namespace>..<name>=<version>");
        let artifact = arg.split(":").collect::<Vec<&str>>();

        let (name, namespace, version) = match artifact.len() {
            2 => (
                artifact[1].to_string(),
                None,
                artifact[2].to_string(),
                ),
            3 => (
                artifact[1].to_string(),
                Some(artifact[0].to_string()),
                artifact[2].to_string(),
                ),
            _ => {
                eprintln!("Package must be defined by <namespace>:<name>:<version>");
                panic!("Package must be defined by <namespace>:<name>:<version>");
            }
        };
        
        let pkg = Package::new(name, namespace, version, PackageKind::Compile);
        
        println!("\r{}", pkg.location.display());
        output
    }
}

impl<'a> DepPath<'_> {
    pub fn new(buildk: &'a BuildK) -> DepPath<'a> {
        DepPath { buildk }
    }
}
