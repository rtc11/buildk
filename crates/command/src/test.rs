use std::path::{Path, PathBuf};

use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::PartialConclusion;

use crate::build_tree::HeaderKt;
use crate::Command;

impl Command {
    pub fn run_tests(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let mut java = ProcessBuilder::new("java");

        let dependencies = config.manifest.dependencies.clone();

        let console_launcher = dependencies
            .iter()
            .filter(|it| it.is_cached())
            .find(|it| it.name.contains("junit-platform-console-standalone"));

        if console_launcher.is_none() {
            output.conclude(PartialConclusion::FAILED);
            println!("missing console logger")
        }

        let dep_jars = dependencies
            .iter()
            .filter(|it| !it.name.contains("junit-platform-console-standalone"))
            .map(|it| it.jar_absolute_path())
            .collect::<Vec<PathBuf>>();

        let mut classpath = vec![
            &config.manifest.project.out.src,
            &config.manifest.project.out.test,
        ];

        classpath.extend(&dep_jars);

        java.cwd(&config.manifest.project.path)
            .jar(&console_launcher.unwrap().jar_absolute_path())
            .classpaths(classpath)
            .args(&["--details", "none"]) //none,flat,tree,verbose
            .test_report(&config.manifest.project.out.test_report);

        let test_files = util::paths::all_files_recursive(vec![], config.manifest.project.test.clone());

        let test_packages = test_files
            .iter()
            .map(Path::new)
            .filter_map(|path| HeaderKt::parse(path).ok())
            .map(|it| it.package)
            .collect::<Vec<String>>();

        for pkg in test_packages.iter() {
            java.args(&["--select-package", &pkg]);
        }

        self.execute(&mut output, &java, 0)
    }
}
