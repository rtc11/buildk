use std::path::PathBuf;

use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::get_kotlinc;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn build_src(&mut self, config: &Config) -> BuildkOutput {
        // let dependency = config.manifest.dependencies.iter().next().unwrap();
        // self.client.download_info(&dependency.name, &dependency.version).unwrap();

        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());
        kotlinc.cwd(&config.manifest.project.path)
            .sources(&config.manifest.build.src)
            .destination(&config.manifest.build.output_src());

        self.execute(&mut output, &kotlinc, 0)
    }

    pub fn build_test(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let mut paths = self.test_libs.clone();
        paths.extend_from_slice(&[
            config.manifest.build.output_src(),
            PathBuf::from("libs/junit-platform-console-standalone-1.9.2.jar"),
        ]);

        let mut kotlinc = ProcessBuilder::new(get_kotlinc());
        kotlinc.cwd(&config.manifest.project.path)
            .sources(&config.manifest.build.test)
            .classpaths(paths)
            .destination(&config.manifest.build.output_test());

        self.execute(&mut output, &kotlinc, 0)
    }
}
