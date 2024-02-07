use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::get_kotlinc;
use util::process_builder::ProcessBuilder;
use util::terminal::Terminal;

use crate::Command;

impl Command {
    pub fn release(
        &self, 
        config: &Config,
        _terminal: &mut Terminal,
    ) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc.cwd(&config.manifest.project.path)
            .include_runtime()
            .destination(&config.manifest.project.out.jar)
            .sources(&config.manifest.project.src);

        self.execute(&mut output, &kotlinc, 0)
    }
}
