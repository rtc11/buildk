use crate::Command;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

impl Command {
    pub fn help(&self, _config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        println!("───────────┬─────────────────────");
        println!("{:<11}│ clean the project", "clean");
        println!("{:<11}│ build the project", "build");
        println!("{:<11}│ test the project", "test");
        println!("{:<11}│ run the project", "run");
        println!("{:<11}│ release the project", "release");
        println!("{:<11}│ fetch the project", "fetch");
        println!("{:<11}│ list the build tree", "tree");
        println!("{:<11}│ print the dependencies", "deps");
        println!("{:<11}│ show the config", "config");
        println!("{:<11}│ print this help", "help");
        println!("───────────┴─────────────────────");
        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
