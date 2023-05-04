use config::config::Config;
use config::dependencies::dependency::DependenciesKind;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;
use util::process_builder::ProcessBuilder;

use crate::Command;

impl Command {
    pub fn run_tests(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let classpath = vec![
            &config.manifest.project.out.src,
            &config.manifest.project.out.test,
        ];

        let mut java = ProcessBuilder::new("java");

        let test_dependencies = config.manifest.dependencies.clone().for_test();
        let junit = test_dependencies.iter()
            .filter(|it| it.is_cached())
            .find(|it|it.name.contains("junit-platform-console-standalone"));

        if junit.is_none() {
            output.conclude(PartialConclusion::FAILED);
            return output
        }

        java.cwd(&config.manifest.project.path)
            .jar(&junit.unwrap().path)
            .classpaths(classpath)
            .args(&["--select-class", "PrefixTest"])
            // .args(&["--select-package", "no.tordly.test"])
            .args(&["--details", "none"])
            .test_report(&config.manifest.project.out.test_report);

        self.execute(&mut output, &java, 0)
    }
}
