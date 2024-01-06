use std::path::PathBuf;

use manifest::config::Config;
use manifest::dependencies::DependenciesKind;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::{get_kotlinc, PartialConclusion};

use crate::{build_tree, Command};

impl Command {
    pub fn build_src(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc
            .cwd(&config.manifest.project.path)
            .destination(&config.manifest.project.out.src);

        match build_tree::sort_by_imports(config) {
            Ok(sorted_src) => {
                let sorted_src = sorted_src
                    .iter()
                    .filter(|file| {
                        let is_cached = self.cache.lock().unwrap().cache_file(file);
                        //println!("file: {:?}: cached=cached, success=new cache: {:?}", file, is_cached);
                        // cached means it was already cached
                        !matches!(is_cached, Ok(PartialConclusion::CACHED))
                    })
                    .collect::<Vec<&PathBuf>>();

                if sorted_src.is_empty() {
                    output.conclude(PartialConclusion::CACHED);
                    return output
                } else {
                    output.conclude(PartialConclusion::SUCCESS);
                    sorted_src.iter().for_each(|src| {
                        kotlinc.sources(src);
                    });
                }
            }
            Err(e) => {
                output.stdout("possible cyclic DAG detected, see stderr".to_owned());
                output.stderr(e.to_string());
                output.conclude(PartialConclusion::FAILED);
                return output;
            }
        }

        self.execute(&mut output, &kotlinc, 0)
    }

    pub fn build_test(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        if !config.manifest.project.test.is_dir() {
            return output
        }

        let project_test_libs = config
            .manifest
            .dependencies
            .clone()
            .for_test()
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
