use crate::Command;
use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

impl Command {
    pub fn help(&self, _config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        println!("───────────┬─────────────────────");
        println!("{:<11}{}", "clean", "│ clean the project");
        println!("{:<11}{}", "build", "│ build the project");
        println!("{:<11}{}", "test", "│ test the project");
        println!("{:<11}{}", "run", "│ run the project");
        println!("{:<11}{}", "release", "│ release the project");
        println!("{:<11}{}", "fetch", "│ fetch the project");
        println!("{:<11}{}", "deps", "│ print the dependencies");
        println!("{:<11}{}", "help", "│ print this help");
        println!("───────────┴─────────────────────");
        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
