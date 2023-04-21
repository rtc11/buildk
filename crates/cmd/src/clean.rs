use anyhow::{bail, Result};
use console::PartialConclusion;
use fs::toml::Config;

pub(crate) fn build_dir(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}", "clean ", config.build.output);

    let result = fs::rm(&std::path::PathBuf::from(&config.build.output));

    match result {
        Ok(_) => {
            println!("{}", PartialConclusion::SUCCESS);
            Ok(String::new())
        },
        Err(err) => {
            println!("{}", PartialConclusion::FAILED);
            bail!(err)
        },
    }
}
