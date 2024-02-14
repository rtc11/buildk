use manifest::config::Config;
use process::kotlin::Kotlin;
use util::buildk_output::BuildkOutput;

use crate::Command;

pub (crate) struct Release<'a> {
    config: &'a Config,
    kotlin: &'a Kotlin<'a>,
}

impl <'a> Command for Release<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("release");

        self.kotlin.builder()
            .source(&self.config.manifest.project.src)
            .include_runtime()
            .workdir(&self.config.manifest.project.path)
            .target(&self.config.manifest.project.out.release)
            .compile(&mut output)
    }
}

impl <'a> Release<'_> {
    pub fn new(config: &'a Config, kotlin: &'a Kotlin) -> Release<'a> {
        Release { config, kotlin }
    }
}
