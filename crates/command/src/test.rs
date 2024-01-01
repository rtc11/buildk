use std::path::PathBuf;

use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;
use util::process_builder::ProcessBuilder;

use crate::Command;

impl Command {
    pub fn run_tests(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new("java");

        let dependencies = config.manifest.dependencies.clone();

        let console_launcher = dependencies
            .iter()
            .filter(|it| it.is_cached())
            .find(|it| it.name.contains("junit-platform-console-standalone"));

        let dep_jars = dependencies
            .iter()
            .filter(|it| !it.name.contains("junit-platform-console-standalone"))
            .map(|it| it.jar_absolute_path())
            .collect::<Vec<PathBuf>>();

        let mut classpath = vec![
            &config.manifest.project.out.src,
            &config.manifest.project.out.test,
        ];

        classpath.extend(&dep_jars);

        if console_launcher.is_none() {
            output.conclude(PartialConclusion::FAILED);
            println!("missing consol logger")
        }

        let test_dir = &config.manifest.project.test.as_path().display().to_string();

        java.cwd(&config.manifest.project.path)
            .jar(&console_launcher.unwrap().jar_absolute_path())
            .classpaths(classpath)
            .args(&["--select-directory", test_dir])
            //.args(&["--select-class", "PrefixTest"])
            .args(&["--select-package", "params"])
            .args(&["--details", "none"])
            .test_report(&config.manifest.project.out.test_report);

        self.execute(&mut output, &java, 0)
    }
}
