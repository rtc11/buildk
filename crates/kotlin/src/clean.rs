use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Kotlin;

impl Kotlin {
    pub fn clean(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let project_output_dir = config.cwd.join(&config.manifest.build.output);

        match std::fs::remove_file(&project_output_dir) {
            Ok(_) => output
                .conclude(PartialConclusion::SUCCESS)
                .stdout(format!("cleared {}", project_output_dir.display()))
                .clone(),
            Err(e) => output
                .conclude(PartialConclusion::FAILED)
                .stderr(format!("failed to clean {} with {}", project_output_dir.display(), e))
                .clone()
        }
    }
}
