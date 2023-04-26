use std::path::PathBuf;

use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn build_src(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(&self.compiler);
        kotlinc.cwd(&config.cwd)
            .sources(&config.manifest.build.src)
            .destination(&config.manifest.build.output_src());

        self.cached_output(&mut output, &kotlinc, 0)
    }

    pub fn build_test(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let mut paths = self.test_libs.clone();
        paths.extend_from_slice(&[
            config.manifest.build.output_src(),
            PathBuf::from("libs/junit-platform-console-standalone-1.9.2.jar"),
        ]);

        let mut kotlinc = ProcessBuilder::new(&self.compiler);
        kotlinc.cwd(&config.cwd)
            .sources(&config.manifest.build.test)
            .classpaths(paths)
            .destination(&config.manifest.build.output_test());

        self.cached_output(&mut output, &kotlinc, 0)
    }
}