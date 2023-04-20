use anyhow::Result;
use console::PartialConclusion;
use fs::toml::Config;

pub(crate) fn build_dir(config: &Config) -> Result<String> {
    print!("▸ {:<7}{}/", "clean ", config.build_dir());

    fs::rm(&std::path::PathBuf::from(config.build_dir()))?;

    println!("{}", PartialConclusion::SUCCESS);

    Ok(String::default())
}
