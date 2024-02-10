use std::path::PathBuf;

use manifest::config::Config;
use manifest::dependencies::Kind;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;

use crate::{RunCmd, Commands};

impl RunCmd for Commands {
    fn run(
        &mut self, 
        config: &Config,
        name: Option<String>,
    ) -> BuildkOutput {
        let mut output = BuildkOutput::new("run");
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

        let main = match name {
            Some(class) => class.to_string() + "Kt",
            None => config.manifest.project.compiled_main_file()
        };

        java.cwd(&config.manifest.project.path)
            .classpaths(classpath)
            .sources(&main);


        /*
        if let Some(stdout) = output.get_stdout() {
            println!("\r\n{stdout}");
        }

        if let Some(stderr) = output.get_stderr() {
            let stderr = stderr.as_red();
            eprintln!("\r\n{stderr}");
        }
        */

        self.execute(&mut output, config, &java, 0)
    }
}

