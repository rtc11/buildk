use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn run(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new("java");

        java.cwd(&config.manifest.project.path)
            .classpath(&config.manifest.build.output_src())
            .sources(&config.manifest.project.main_class());

        self.execute(&mut output, &java, 0)
    }
}
