use std::process::Command;

use anyhow::{Context, Result};

use console::PartialConclusion;
use fs::toml::{BuildCache, Config};

use crate::{KOTLINC, log_result};

pub(crate) fn src(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "build ", config.build.src);

    let mut cache = get_cache(config);
    if cache.missing_src() {
        let output = Command::new(KOTLINC)
            .current_dir(&config.project.dir)
            .arg(&config.build.src)
            .args(["-d", &config.build.output_src()])
            .output()
            .with_context(|| PartialConclusion::FAILED)?;

        let result = log_result(output);
        if let Ok(_) = &result {
            let dir = format!("{}/{}", config.project.dir, config.build.output_src());
            cache.set_src(fs::ls(&dir).expect("output_src dir not found"));
            save_cache(&config, &cache);
        }
        result
    } else {
        println!("{}", PartialConclusion::CACHED);
        Ok(String::new())
    }
}

pub(crate) fn test(config: &Config) -> Result<String> {
    print!("▸ {:<7}{:<7}", "build ", config.build.test);

    let mut cache = get_cache(config);
    if cache.missing_test() {
        let classpath = [
            &config.build.output_src(),
            "../kotlinc/lib/kotlin-test-junit5.jar",
            "../kotlinc/lib/kotlin-test.jar",
            "libs/junit-platform-console-standalone-1.9.2.jar",
        ].join(":");

        let output = Command::new(KOTLINC)
            .current_dir(&config.project.dir)
            .arg(&config.build.test)
            .args(["-cp", &classpath])
            .args(["-d", &config.build.output_test()])
            .output()
            .with_context(|| "failed  to build tests")?;

        let result = log_result(output);
        if let Ok(_) = &result {
            let dir = format!("{}/{}", config.project.dir, config.build.output_test());
            cache.set_test(fs::ls(&dir).expect("output_test dir not found"));
            save_cache(&config, &cache);
        }
        result
    } else {
        println!("{}", PartialConclusion::CACHED);
        Ok(String::new())
    }
}

fn get_cache(config: &Config) -> BuildCache {
    let dir = format!("{}/{}", config.project.dir, config.build.cache());
    fs::toml::read_file(&dir).unwrap_or_default()
}

fn save_cache(config: &Config, cache: &BuildCache) {
    let dir = format!("{}/{}", config.project.dir, config.build.cache());
    fs::toml::write_file(&dir, cache).expect("Failed.")
}
