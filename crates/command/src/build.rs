use std::path::PathBuf;

use config::config::Config;
use config::dependencies::dependency::DependenciesKind;
use util::buildk_output::BuildkOutput;
use util::{get_kotlinc, PartialConclusion};
use util::paths::all_files_recursive;
use util::process_builder::ProcessBuilder;

use crate::Command;

impl Command {
    pub fn build_src(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        let src_files = all_files_recursive(vec![], config.manifest.project.src.clone());
        let files_to_build = src_files.iter().filter(|file| {
            matches!(self.cache.lock().unwrap().cache_file(file), Ok(PartialConclusion::SUCCESS))
        }).collect::<Vec<&PathBuf>>();

        kotlinc.cwd(&config.manifest.project.path)
            .destination(&config.manifest.project.out.src);

        if files_to_build.is_empty() {
            output.conclude(PartialConclusion::CACHED);
            return output
        }
        files_to_build.iter().for_each(|file| {
           kotlinc.sources(file);
        });

        self.execute(&mut output, &kotlinc, 0)
    }

    pub fn build_test(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let project_test_libs = config.manifest.dependencies
            .clone()
            .for_test()
            .into_iter()
            .filter(|dependency| dependency.is_cached())
            .map(|dependency| dependency.jar_path())
            .collect::<Vec<PathBuf>>();

        let mut classpath = vec![&config.manifest.project.out.src];
        classpath.extend(project_test_libs.iter());
        classpath.extend(self.test_libs.iter());
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());
        kotlinc.cwd(&config.manifest.project.path)
            .sources(&config.manifest.project.test)
            .classpaths(classpath)
            .destination(&config.manifest.project.out.test);

        self.execute(&mut output, &kotlinc, 0)
    }
}
