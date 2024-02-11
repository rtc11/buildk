use std::fs::remove_dir_all;
use std::path::Path;

use cache::cache::Cache;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Command;

const OS_2_ERROR: &str = "No such file or directory (os error 2)";

pub (crate) struct Clean<'a> {
    config: &'a Config,
    cache: &'a mut Cache,
}

impl <'a> Command for Clean<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("clean");

        let out_dir = &self.config.manifest.project.out.path;

        match remove_dir_all(out_dir) {
            Ok(_) => self.cleaned(&mut output, out_dir),
            Err(e) if e.to_string() == *OS_2_ERROR => self.cleaned(&mut output, out_dir),
            Err(e) => self.failed(&mut output, out_dir, e)
        }
    }


}

impl <'a> Clean<'_> {
    pub fn new(config: &'a Config, cache: &'a mut Cache) -> Clean<'a> {
        Clean { config, cache }
    }

    fn cleaned(&mut self, output: &mut BuildkOutput, dir: &Path) -> BuildkOutput {
        self.cache.invalidate(); // todo: does this work? is it mutable?

        output
            .conclude(PartialConclusion::SUCCESS)
            .stdout(format!("{} cleaned.", dir.display()))
            .clone()
    }

    fn failed(&self, output: &mut BuildkOutput, dir: &Path, err: std::io::Error) -> BuildkOutput {
        output
            .conclude(PartialConclusion::FAILED)
            .stderr(format!("failed to clean {} with {}", dir.display(), err))
            .clone()
    }
}

