use anyhow::{bail, Result};

use console::PartialConclusion;
use fs::toml::Config;

pub(crate) fn build_dir(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "clean ", config.build.output);

    let project_output_dir = format!("{}/{}", config.project.dir, config.build.output);
    let result = fs::rm(&project_output_dir);

    match result {
        Ok(_) => {
            println!("{}", PartialConclusion::SUCCESS);
            Ok(String::new())
        }
        Err(err) => {
            println!("{}", PartialConclusion::FAILED);
            bail!(err)
        }
    }
}
