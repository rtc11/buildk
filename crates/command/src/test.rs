use std::path::{PathBuf, Path};

use manifest::config::Config;
use util::buildk_output::BuildkOutput;
use util::process_builder::ProcessBuilder;
use util::PartialConclusion;
use util::terminal::Terminal;

use crate::build_tree::HeaderKt;
use crate::Command;

impl Command {
    pub fn run_tests(
        &self, 
        config: &Config,
        _terminal: &mut Terminal,
    ) -> BuildkOutput {
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
            .args(&["--details", "tree"]) //none,flat,tree,verbose
            .args(&["--exclude-engine", "junit-vintage"]) //engine:junit-platform-suite
            .args(&["--exclude-engine", "junit-platform-suite"]) //engine:junit-platform-suite
            .test_report(&config.manifest.project.out.test_report);

        if let Ok(test_files) = util::paths::all_files_recursive(vec![], config.manifest.project.test.clone()){
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

        /*
        if let Some(stdout) = output.get_stdout() {
            println!("\r\n{stdout}");
        }    
        */

        self.execute(&mut output, &java, 0)
    }
}

