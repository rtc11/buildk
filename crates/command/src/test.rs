use std::path::Path;

use manifest::config::Config;
use manifest::dependencies::DependenciesTools;
use manifest::manifest::Manifest;
use process::java::Java;
use util::buildk_output::BuildkOutput;
use util::PartialConclusion;

use crate::Command;
use crate::tree::HeaderKt;

pub (crate) struct Test<'a> {
    config: &'a Config,
    java: &'a Java<'a>,
}

impl <'a> Command for Test<'a> {
    type Item = String;

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("test");

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest)
            .expect("no buildk.toml found.");

        let console_launcher = match manifest.dependencies.junit_platform() {
            Some(dep) => dep.jar_absolute_path(),
            None => {
                return output
                    .conclude(PartialConclusion::FAILED)
                    .stderr("missing junit platform dependency".to_owned()).to_owned();
            }
        };

        let test_deps = manifest.dependencies.test_deps();
        let test_deps_cp = test_deps.iter().map(|dep| dep.jar_absolute_path()).collect::<Vec<_>>();
        let junit_cp = manifest.dependencies.junit_runner().map(|dep| dep.jar_absolute_path()).expect("missing junit");

        let mut classpath = vec![
            &manifest.project.out.src,
            &manifest.project.out.test,
            &junit_cp,
        ];

        classpath.extend(&test_deps_cp);

        let mut java = self.java.builder();
        java.workdir(&manifest.project.path)
            .classpath(classpath)
            .jar(&console_launcher)
            .test_report(&manifest.project.out.test_report)
            .args(&["--details", "tree"])
            .args(&["--exclude-engine", "junit-vintage"])
            .args(&["--exclude-engine", "junit-platform-suite"]);

        if let Ok(test_files) = util::paths::all_files_recursive(vec![], manifest.project.test.clone()){
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

        java.run(&mut output)
    }
}

impl <'a> Test <'_> {
    pub fn new(config: &'a Config, java: &'a Java) -> Test<'a> {
        Test { config, java }
    }
}

