use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn run_tests(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let classpath = vec![
            config.manifest.build.output_src(),
            config.manifest.build.output_test(),
        ];

        let mut java = ProcessBuilder::new("java");
        java.cwd(&config.manifest.project.path)
            .args(&["-jar", "libs/junit-platform-console-standalone-1.9.2.jar"])
            .classpaths(classpath)
            .args(&["--select-class", "PrefixTest"])
            // .args(&["--select-package", "no.tordly.test"])
            .args(&["--details", "none"])
            .test_report(&config.manifest.build.output_test_report());

        self.execute(&mut output, &java, 0)
    }
}
