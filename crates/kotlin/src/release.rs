use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::get_kotlinc;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn release(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new(get_kotlinc());

        java.cwd(&config.manifest.project.path)
            .sources(&config.manifest.build.src)
            .include_runtime()
            .destination(&config.manifest.build.output_target());

        self.execute(&mut output, &java, 0)
    }
}
