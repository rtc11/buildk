use std::path::PathBuf;

use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::{get_kotlinc, PartialConclusion};

use crate::{build_tree, Command};

const DEBUG: bool = false;

impl Command {

    pub fn build_src(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc
            .cwd(&config.manifest.project.path)
            .destination(&config.manifest.project.out.src);

        let extra_fingerprints = match build_tree::sort_by_imports(config) {
            Ok(sorted_src) => {
                let sorted_src = sorted_src
                    .iter()
                    .filter(|file| {
                        let has_changes = match self.cache.lock().unwrap().cache_file(file) {
                            Ok(PartialConclusion::CACHED) => false,
                            _ => true,
                        };

                        if DEBUG {
                            println!("\r {} {}", if has_changes { "compile" } else { "cached" }, file.display())
                        }

                        has_changes

                    })
                    .collect::<Vec<&PathBuf>>();

                if sorted_src.is_empty() {
                    output.conclude(PartialConclusion::CACHED);
                    return output
                } else {
                    output.conclude(PartialConclusion::SUCCESS);
                    sorted_src
                        .iter()
                        .map(|src| {
                            // extra fingerprints is used to check if the kotlinc command should be
                            // rerun (its files have been modified)
                            let fingerprint = cache::file_fingerprint(src).expect("failed to fingerprint file");

                            kotlinc.sources(src);

                            if DEBUG {
                                println!("compiling {}", src.display());
                            }

                            fingerprint
                        }).collect::<Vec<_>>()
                }
            }
            Err(e) => {
                output.stdout("possible cyclic DAG detected, see stderr".to_owned());
                output.stderr(e.to_string());
                output.conclude(PartialConclusion::FAILED);
                return output;
            }
        };

        let extra = extra_fingerprints.into_iter().reduce(|a, b| (a + b)).expect("failed to reduce fingerprints");

        self.execute(&mut output, &kotlinc, extra)
    }

    pub fn build_test(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        // return if no tests are configured
        if !config.manifest.project.test.is_dir() {
            return output
        }

        let project_test_libs = config
            .manifest
            .dependencies
            .clone()
            .into_iter()
            .filter(|dependency| dependency.is_cached())
            .map(|dependency| dependency.jar_absolute_path())
            .collect::<Vec<PathBuf>>();

        let mut classpath = vec![&config.manifest.project.out.src];
        classpath.extend(project_test_libs.iter());
        classpath.extend(self.test_libs.iter());

        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc
            .cwd(&config.manifest.project.path)
            .sources(&config.manifest.project.test)
            .classpaths(classpath)
            .destination(&config.manifest.project.out.test);

        self.execute(&mut output, &kotlinc, 0)
    }
}
