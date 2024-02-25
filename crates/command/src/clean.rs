use std::fs::remove_dir_all;
use std::path::Path;

use cache::cache::Cache;
use manifest::config::Config;
use manifest::manifest::Manifest;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::{Command, CleanSet};

const OS_2_ERROR: &str = "No such file or directory (os error 2)";

pub (crate) struct Clean<'a> {
    config: &'a Config,
}

impl <'a> Command for Clean<'a> {
    type Item = CleanSet;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("clean");
        let arg = arg.expect("arg should have default value == All");

        match arg {
            CleanSet::Src => self.clean_src(&mut output),
            CleanSet::Test => self.clean_test(&mut output),
            CleanSet::Release => self.clean_release(&mut output),
            CleanSet::All => self.clean_all(&mut output)
        }
    }
}

impl <'a> Clean<'_> {
    pub fn new(config: &'a Config) -> Clean<'a> {
        Clean { config }
    }

    fn clean_src(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest).expect("manifest");

        let path = &manifest.project.out.src;
        self.delete(output, path)
    }

    fn clean_test(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest).expect("manifest");
        let path = &manifest.project.out.test;
        self.delete(output, path)
    }

    fn clean_release(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest).expect("manifest");
        let path = &manifest.project.out.release;
        self.delete(output, path)
    }

    fn clean_all(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest).expect("manifest");
        let path = &manifest.project.out.path;
        self.delete(output, path) 
    }

    fn delete(&mut self, output: &mut BuildkOutput, out_dir: &Path) -> BuildkOutput {
        match remove_dir_all(out_dir) {
            Ok(_) => self.cleaned(output, out_dir),
            Err(e) if e.to_string() == *OS_2_ERROR => self.cleaned(output, out_dir),
            Err(e) => self.failed(output, out_dir, e)
        }
    }

    fn cleaned(&mut self, output: &mut BuildkOutput, dir: &Path) -> BuildkOutput {
        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest).expect("manifest");
        let mut cache = Cache::load(&manifest.project.out.cache);
        cache.invalidate();

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
