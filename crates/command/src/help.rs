use crate::Command;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;
use util::terminal::Terminal;

impl Command {
    pub fn help(
        &self, 
        _config: &Config,
        _terminal: &mut Terminal,
    ) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let mut display = String::new();
        display.push_str("───────────┬─────────────────────\n");
        display.push_str(&format!("{:<11}│ clean the project\n", "clean"));
        display.push_str(&format!("{:<11}│ build the project\n", "build"));
        display.push_str(&format!("{:<11}│ test the project\n", "test"));
        display.push_str(&format!("{:<11}│ run the project\n", "run"));
        display.push_str(&format!("{:<11}│ release the project\n", "release"));
        display.push_str(&format!("{:<11}│ fetch the project\n", "fetch"));
        display.push_str(&format!("{:<11}│ list the build tree\n", "tree"));
        display.push_str(&format!("{:<11}│ print the dependencies\n", "deps"));
        display.push_str(&format!("{:<11}│ show the config\n", "config"));
        display.push_str(&format!("{:<11}│ print this help\n", "help"));
        display.push_str(&format!("───────────┴─────────────────────"));

        println!("\r{}", display);
        output.conclude(PartialConclusion::SUCCESS);
        output
    }
}
