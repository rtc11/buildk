use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn run(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new("java");
        java.cwd(&config.cwd)
            .sources(&config.manifest.project.main_class())
            .classpath(&config.manifest.build.src);

        self.cached_output(&mut output, &java, 0)
    }
}
