use manifest::config::Config;
use manifest::manifest::Manifest;
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
        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest)
            .expect("no buildk.toml found.");

        self.kotlin.builder()
            .source(&manifest.project.src)
            .include_runtime()
            .workdir(&manifest.project.path)
            .target(&manifest.project.out.release)
            .compile(&mut output)
    }
}

impl <'a> Release<'_> {
    pub fn new(config: &'a Config, kotlin: &'a Kotlin) -> Release<'a> {
        Release { config, kotlin }
    }
}
