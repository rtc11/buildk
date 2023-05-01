use config::config::Config;
use util::PartialConclusion;
use util::buildk_output::BuildkOutput;

use crate::Kotlin;

impl Kotlin {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        config.manifest.dependencies.iter().for_each(|dependency| {
            let info = self.client.dependency_info(&dependency.name, &dependency.version).unwrap();
            self.client.download(info).expect("downloaded")
        });

        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
