use std::path::Path;

use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Kotlin;

const OS_2_ERROR: &str = "No such file or directory (os error 2)";

impl Kotlin {
    pub fn clean(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let output_dir = &config.manifest.project.out.path;
        match std::fs::remove_dir_all(output_dir) {
            Ok(_) => self.cleaned(&mut output, output_dir),
            Err(e) if e.to_string() == *OS_2_ERROR => self.cleaned(&mut output, output_dir),
            Err(e) => self.failed(&mut output, output_dir, e)
        }
    }

    fn cleaned(&mut self, output: &mut BuildkOutput, path: &Path) -> BuildkOutput {
        self.invalidate_cache();
        output
            .conclude(PartialConclusion::SUCCESS)
            .stdout(format!("{} cleaned.", path.display()))
            .clone()
    }

    fn failed(&mut self, output: &mut BuildkOutput, path: &Path, e: std::io::Error) -> BuildkOutput {
        output
            .conclude(PartialConclusion::FAILED)
            .stderr(format!("failed to clean {} with {}", path.display(), e))
            .clone()
    }
}
