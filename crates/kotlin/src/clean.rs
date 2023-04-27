use std::path::PathBuf;

use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Kotlin;

const OS_2_ERROR: &'static str = "No such file or directory (os error 2)";

impl Kotlin {
    pub fn clean(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let project_output_dir = config.cwd.join(&config.manifest.build.output);

        match std::fs::remove_dir_all(&project_output_dir) {
            Ok(_) => self.cleaned(&mut output, &project_output_dir),
            Err(e) if e.to_string() == OS_2_ERROR.to_string() => self.cleaned(&mut output, &project_output_dir),
            Err(e) => self.failed(&mut output, &project_output_dir, e).clone()
        }
    }

    fn cleaned(&mut self, output: &mut BuildkOutput, path: &PathBuf) -> BuildkOutput {
        self.invalidate_cache();
        output
            .conclude(PartialConclusion::SUCCESS)
            .stdout(format!("{} cleaned.", path.display()))
            .clone()
    }

    fn failed(&mut self, output: &mut BuildkOutput, path: &PathBuf, e: std::io::Error) -> BuildkOutput {
        output
            .conclude(PartialConclusion::FAILED)
            .stderr(format!("failed to clean {} with {}", path.display(), e))
            .clone()
    }
}
