use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

use crate::Command;

impl Command {
    pub fn run(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new("java");

        java.cwd(&config.manifest.project.path)
            .classpath(&config.manifest.project.out.src)
            .sources(&config.manifest.project.compiled_main_file());

        self.execute(&mut output, &java, 0)
    }
}
