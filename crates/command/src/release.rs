use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::get_kotlinc;
use util::process_builder::ProcessBuilder;

use crate::{Commands, ReleaseCmd};

impl ReleaseCmd for Commands {
    fn release(
        &mut self, 
        config: &Config,
    ) -> BuildkOutput {
        let mut output = BuildkOutput::new("release");
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc.cwd(&config.manifest.project.path)
            .include_runtime()
            .destination(&config.manifest.project.out.release)
            .sources(&config.manifest.project.src);

        //todo cache
        self.execute(&mut output, config, &kotlinc, 0)
    }
}
