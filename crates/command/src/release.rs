use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::get_kotlinc;
use util::process_builder::ProcessBuilder;

use crate::Command;

impl Command {
    pub fn release(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new(get_kotlinc());

        java.cwd(&config.manifest.project.path)
            .sources(&config.manifest.project.src)
            .include_runtime()
            .destination(&config.manifest.project.out.jar);

        self.execute(&mut output, &java, 0)
    }
}
