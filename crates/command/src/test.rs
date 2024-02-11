use std::path::{PathBuf, Path};

use cache::cache::Cache;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::process_builder::ProcessBuilder;
use util::PartialConclusion;

use crate::Command;
use crate::tree::HeaderKt;

pub (crate) struct Test<'a> {
    config: &'a Config,
    cache: &'a mut Cache,
}

impl <'a> Command for Test<'a> {
    type Item = String;

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("test");
        let mut java = ProcessBuilder::new("java");

        let dependencies = self.config.manifest.dependencies.clone();

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
            &self.config.manifest.project.out.src,
            &self.config.manifest.project.out.test,
        ];

        classpath.extend(&dep_jars);

        java.cwd(&self.config.manifest.project.path)
            .jar(&console_launcher.unwrap().jar_absolute_path())
            .classpaths(classpath)
            .args(&["--details", "tree"]) //none,flat,tree,verbose
            .args(&["--exclude-engine", "junit-vintage"]) //engine:junit-platform-suite
            .args(&["--exclude-engine", "junit-platform-suite"]) //engine:junit-platform-suite
            .test_report(&self.config.manifest.project.out.test_report);

        if let Ok(test_files) = util::paths::all_files_recursive(vec![], self.config.manifest.project.test.clone()){
            let test_packages = test_files
                .iter()
                .map(Path::new)
                .filter_map(|path| HeaderKt::parse(path).ok())
                .map(|it| it.package)
                .collect::<Vec<String>>();

            for pkg in test_packages.iter() {
                java.args(&["--select-package", &pkg]);
            }
        }

        self.execute_with_cache(&mut output, &java)
    }
}

impl <'a> Test <'_> {
    pub fn new(config: &'a Config, cache: &'a mut Cache) -> Test<'a> {
        Test { config, cache }
    }

    fn execute_with_cache(
        &mut self,
        output: &mut BuildkOutput,
        cmd: &ProcessBuilder,
    ) -> BuildkOutput {
        match self.cache.cache_command(cmd, 0) {
            Ok(cache_res) => {
                output
                    .conclude(cache_res.conclusion)
                    .stdout(cache_res.stdout.unwrap_or("".to_owned()))
                    .status(cache_res.status);

                if let Some(stderr) = cache_res.stderr {
                    output
                        .conclude(PartialConclusion::FAILED)
                        .stderr(stderr);
                }

                output.to_owned()
            }

            Err(err) => {
                let err = err.to_string().as_red();

                println!("\r{err:#}");

                output
                    .conclude(PartialConclusion::FAILED)
                    .stderr(err.to_string())
                    .to_owned()
            },
        }
    }
}
