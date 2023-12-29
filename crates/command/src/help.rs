use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;
use crate::Command;

impl Command {

    pub fn help(&self, _config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        println!("build, clean, fetch, help, deps, release, run, test");
        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
