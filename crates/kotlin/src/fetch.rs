use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Kotlin;

impl Kotlin {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        config.manifest.dependencies.iter().for_each(|dependency| {
            let info = self.client.dependency_info(&dependency.name, &dependency.version).unwrap();
            match info.is_cached() {
                true => {
                    println!("found dependency in cache: {}", &info.filename);
                    output.conclude(PartialConclusion::CACHED);
                },
                false => match self.client.download(&info) {
                    Ok(_) => {
                        println!("downloaded and cached dependency: {}", &info.filename);
                        output.conclude(PartialConclusion::SUCCESS);
                    },
                    Err(_) => {
                        println!("Failed to dwnload dependency: {}", &info.filename);
                        output.conclude(PartialConclusion::FAILED);
                    },
                }
            }
        });

        output
    }
}
