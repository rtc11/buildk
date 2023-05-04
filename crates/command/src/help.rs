use config::config::Config;
use util::buildk_output::BuildkOutput;
use crate::Command;

impl Command {

    pub fn help(&self, _config: &Config) -> BuildkOutput {
        println!("build, clean, fetch, help, list, release, run, test");

        BuildkOutput::default()
    }
}
