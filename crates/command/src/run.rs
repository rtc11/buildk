use std::path::PathBuf;

use cache::cache::Cache;
use manifest::config::Config;
use manifest::dependencies::Kind;
use util::PartialConclusion;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::process_builder::ProcessBuilder;

use crate::Command;

pub (crate) struct Run<'a> {
    config: &'a Config,
    cache: &'a mut Cache,
}

impl <'a> Command for Run<'a> {
    type Item = String;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("run");
        let mut java = ProcessBuilder::new("java");

        let dependencies = self.config
            .manifest
            .dependencies
            .iter()
            .filter(|dep| dep.kind != Kind::Test)
            .filter(|it| !it.name.contains("junit-platform-console-standalone"))
            .map(|dep| dep.jar_absolute_path())
            .collect::<Vec<PathBuf>>();

        let mut classpath = vec![
            &self.config.manifest.project.out.src,
            &self.config.manifest.project.src,
        ];

        classpath.extend(dependencies.iter());

        let main = match arg {
            Some(class) => class.to_string() + "Kt",
            None => self.config.manifest.project.compiled_main_file()
        };

        java.cwd(&self.config.manifest.project.path)
            .classpaths(classpath)
            .sources(&main);

        self.execute_with_cache(&mut output, &java)
    }
}

impl <'a> Run<'_> {
    pub fn new(config: &'a Config, cache: &'a mut Cache) -> Run<'a> {
        Run { config, cache }
    }

    fn execute_with_cache(
        &mut self,
        output: &mut BuildkOutput,
        cmd: &ProcessBuilder,
    ) -> BuildkOutput {
        match self.cache.cache_command(cmd, 0) {
            Ok(cache_res) => {
                output
                    .conclude(cache_res.conclusion)
                    .stdout(cache_res.stdout.unwrap_or("".to_owned()))
                    .status(cache_res.status);

                if let Some(stderr) = cache_res.stderr {
                    output
                        .conclude(PartialConclusion::FAILED)
                        .stderr(stderr);
                }

                output.to_owned()
            }

            Err(err) => {
                let err = err.to_string().as_red();

                println!("\r{err:#}");

                output
                    .conclude(PartialConclusion::FAILED)
                    .stderr(err.to_string())
                    .to_owned()
            },
        }
    }
}

