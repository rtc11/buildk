use cache::cache::Cache;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::{get_kotlinc, PartialConclusion};
use util::process_builder::ProcessBuilder;

use crate::Command;

pub (crate) struct Release<'a> {
    config: &'a Config,
    cache: &'a mut Cache,
}

impl <'a> Command for Release<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("release");
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc.cwd(&self.config.manifest.project.path)
            .include_runtime()
            .destination(&self.config.manifest.project.out.release)
            .sources(&self.config.manifest.project.src);

        self.execute_with_cache(&mut output, &kotlinc)
    }
}

impl <'a> Release<'_> {
    pub fn new(config: &'a Config, cache: &'a mut Cache) -> Release<'a> {
        Release { config, cache }
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
