use std::path::PathBuf;

use cache::cache::Cache;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::colorize::Colorize;
use util::process_builder::ProcessBuilder;
use util::{get_kotlinc, PartialConclusion, get_kotlin_home};

use crate::tree::Tree;
use crate::{Set, Command};

pub (crate) struct Build<'a> {
    config: &'a Config,
    cache: &'a mut Cache,
    tree: &'a Tree,
}

impl <'a> Command for Build<'a> {
    type Item = Set;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("build");
        match arg {
            Some(Set::Src) => self.build_src(&mut output),
            Some(Set::Test) => self.build_test(&mut output),
            _ => {
                let mut output = self.build_src(&mut output);
                self.build_test(&mut output)
            }
        }
    }
}

impl <'a> Build<'_> {
    pub fn new(config: &'a Config, cache: &'a mut Cache, tree: &'a Tree) -> Build<'a> {
        Build { config, cache, tree }
    }

    fn is_not_cached(&mut self, file: &PathBuf) -> bool {
        let conclusion = self.cache.cache_file(file);
        !matches!(conclusion, Ok(PartialConclusion::CACHED))
    }

    fn build_src(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());
        
        kotlinc
            .cwd(&self.config.manifest.project.path)
            .destination(&self.config.manifest.project.out.src);

        let build_tree = self.tree.get_sorted_tree().expect("Failed to get sorted build tree");
        let changed_files: Vec<&PathBuf> = build_tree.iter().filter(|file| self.is_not_cached(file)).collect();

        if changed_files.is_empty() {
            output.conclude(PartialConclusion::CACHED);
            return output.to_owned()
        }

        let extra_fingerprints = changed_files.iter().map(|src| {
            kotlinc.sources(src);
            cache::file_fingerprint(src).expect("Faile to create extra fingerprint")
        }).reduce(|a, b| a + b);

        let extra_fingerprint = match extra_fingerprints {
            Some(fingerprint) => fingerprint,
            None => {
                output.stdout("possible cyclic DAG detected, see stderr".to_owned());
                output.stderr("Failed to create extra fingerprint".to_owned());
                output.conclude(PartialConclusion::FAILED);
                return output.to_owned()
            }
        };

        self.execute_with_cache(output, &kotlinc, extra_fingerprint)
    }

    fn build_test(&mut self, output: &mut BuildkOutput) -> BuildkOutput {
        let config = self.config;

        // return if no tests are configured
        if !config.manifest.project.test.is_dir(){
            return output.to_owned()
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
        let kotlin_home = get_kotlin_home();
        let test_libs = vec![
            kotlin_home.join("libexec/lib/kotlin-test-junit5.jar"),
            kotlin_home.join("libexec/lib/kotlin-test.jar"),
        ];

        classpath.extend(project_test_libs.iter());
        classpath.extend(test_libs.iter());

        let mut kotlinc = ProcessBuilder::new(get_kotlinc());

        kotlinc
            .cwd(&config.manifest.project.path)
            .sources(&config.manifest.project.test)
            .classpaths(classpath)
            .destination(&config.manifest.project.out.test);

        self.execute_with_cache(output, &kotlinc, 0)
    }
    
    fn execute_with_cache(
        &mut self,
        output: &mut BuildkOutput,
        cmd: &ProcessBuilder,
        extra: u64,
    ) -> BuildkOutput {
        match self.cache.cache_command(cmd, extra) {
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

