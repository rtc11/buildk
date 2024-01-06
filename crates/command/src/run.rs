use std::path::PathBuf;

use manifest::config::Config;
use manifest::dependencies::Kind;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::process_builder::ProcessBuilder;

use crate::Command;

impl Command {
    pub fn run(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new("java");

        let dependencies = &config
            .manifest
            .dependencies
            .iter()
            .filter(|dep| dep.kind != Kind::Test)
            .filter(|it| !it.name.contains("junit-platform-console-standalone"))
            .map(|dep| dep.jar_absolute_path())
            .collect::<Vec<PathBuf>>();

        let mut classpath = vec![
            &config.manifest.project.out.src,
            &config.manifest.project.src,
        ];

        classpath.extend(dependencies.iter());

        java.cwd(&config.manifest.project.path)
            .classpaths(classpath)
            .sources(&config.manifest.project.compiled_main_file());

        let output = self.execute(&mut output, &java, 0);

        if let Some(stdout) = output.get_stdout() {
            println!("\r\n{stdout}");
        }

        if let Some(stderr) = output.get_stderr() {
            let stderr = stderr.as_red();
            eprintln!("\r\n{stderr}");
        }

        output
    }
}
