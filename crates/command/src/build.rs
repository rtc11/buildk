use std::path::PathBuf;

use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::{get_kotlinc, PartialConclusion, get_kotlin_home};

use crate::tree::Tree;
use crate::{Commands, Set, BuildCmd};

const DEBUG: bool = false;

impl BuildCmd for Commands {

    fn build(&mut self, config: &Config, source: Set) -> BuildkOutput {
        let mut output = BuildkOutput::new("build");

        match source {
            Set::Src => self.build_src(&mut output, config),
            Set::Test => self.build_test(&mut output, config),
            Set::All => {
                let mut output = self.build_src(&mut output, config);
                self.build_test(&mut output, config)
            }
        }
    }
}

impl Commands {
    fn build_src(&self, output: &mut BuildkOutput, config: &Config) -> BuildkOutput {
        let mut kotlinc = ProcessBuilder::new(get_kotlinc());
        let mut cache = self.load_cache(config);
        
        kotlinc
            .cwd(&config.manifest.project.path)
            .destination(&config.manifest.project.out.src);

        let mut tree = Tree::new(config);
    
        let extra_fingerprints = match tree.sort_by_imports() {
            Ok(_) => {
                let sorted_src = tree.files
                    .iter()
                    .filter(|file| {
                        let has_changes = !matches!(cache.cache_file(file), Ok(PartialConclusion::CACHED));
                        if DEBUG {
                            println!("\r {} {}", if has_changes { "compile" } else { "cached" }, file.display())
                        }

                        has_changes
                    })
                    .collect::<Vec<&PathBuf>>();

                if sorted_src.is_empty() {
                    output.conclude(PartialConclusion::CACHED);
                    return output.to_owned()
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
                return output.to_owned()
            }
        };

        let extra = extra_fingerprints.into_iter().reduce(|a, b| (a + b)).expect("failed to reduce fingerprints");

        self.execute(output, config, &kotlinc, extra)
    }

    fn build_test(&self, output: &mut BuildkOutput, config: &Config) -> BuildkOutput {

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

        self.execute(output, config, &kotlinc, 0)
    }
}

