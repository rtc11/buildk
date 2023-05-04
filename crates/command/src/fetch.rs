use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Command;

impl Command {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        config.manifest.dependencies.iter().for_each(|dependency| {
            match dependency.is_cached() {
                true => {
                    // println!("found dependency in cache: {}:{}", dependency.name, dependency.version);
                    output.conclude(PartialConclusion::CACHED);
                },
                false => match self.client.download(dependency) {
                    Ok(_) => {
                        // println!("downloaded and cached dependency: {}:{}", dependency.name, dependency.version);
                        output.conclude(PartialConclusion::SUCCESS);
                    },
                    Err(_) => {
                        // println!("Failed to download dependency: {}:{}", dependency.name, dependency.version);
                        output.conclude(PartialConclusion::FAILED);
                    },
                }
            }
        });

        output
    }
}