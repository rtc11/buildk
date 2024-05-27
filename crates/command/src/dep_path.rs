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

        // let manifest = <Option<Manifest> as Clone>::clone(&self.buildk.manifest)
        //     .expect("no buildk.toml found.");

        let arg = arg.expect("missing arg <namespace>..<name>=<version>");
        let artifact = arg.split("=").collect::<Vec<&str>>();
        if artifact.len() != 2 {
            panic!("unexpected artifact name. Missing artifact or version: <namespace>..<name>=<version>");
        }
        
        let (artifact, version) = (artifact[0], artifact[1].to_string());

        let artifact = artifact.split("..").collect::<Vec<&str>>();
        let (name, namespace) = match artifact.len() {
            1 => {
                let name = artifact[0].to_string();
                (name, None)
            }
            2 => {
                let name = artifact[1].to_string();
                let namespace = artifact[0].to_string();
                (name, Some(namespace))
            }
            _ => panic!("unexpected artifact name"),
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
