use std::path::PathBuf;

use config::config::Config;
use config::dependencies::dependency::DependenciesKind;
use util::buildk_output::BuildkOutput;
use util::get_kotlinc;
use util::process_builder::ProcessBuilder;

use crate::Kotlin;

impl Kotlin {
    pub fn build_src(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());
        kotlinc.cwd(&config.manifest.project.path)
            .sources(&config.manifest.project.src)
            .destination(&config.manifest.project.out.src);

        self.execute(&mut output, &kotlinc, 0)
    }

    pub fn build_test(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        let project_test_libs = config.manifest.dependencies
            .clone()
            .for_test()
            .into_iter()
            .filter(|dependency| dependency.is_cached())
            .map(|dependency| dependency.path)
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
