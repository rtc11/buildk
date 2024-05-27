use std::path::Path;

use manifest::config::BuildK;
use manifest::Manifest;
use process::java::Java;
use util::buildk_output::BuildkOutput;

use crate::tree::HeaderKt;
use crate::Command;

pub(crate) struct Test<'a> {
    buildk: &'a BuildK,
    java: &'a Java<'a>,
}

impl<'a> Command for Test<'a> {
    type Item = String;

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("test");

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.buildk.manifest)
            .expect("no buildk.toml found.");

        let console_launcher = manifest
            .test_deps
            .pkgs
            .iter()
            .find(|pkg| pkg.name == "junit-platform-console-standalone")
            .map(|pkg| pkg.jar_absolute_path())
            .expect("missing junit-platform-console-standalone");

        let test_deps = manifest
            .test_deps
            .pkgs
            .iter()
            .map(|pkg| pkg.jar_absolute_path())
            .collect::<Vec<_>>();

        let junit = manifest
            .test_deps
            .pkgs
            .iter()
            .find(|pkg| pkg.name == "junit-jupiter-api")
            .map(|pkg| pkg.jar_absolute_path())
            .expect("missing junit");

        let out_paths = &manifest.project.out_paths();
        let mut classpath = vec![&out_paths.src, &out_paths.test, &junit];

        classpath.extend(&test_deps);

        let mut java = self.java.builder();
        java.workdir(&manifest.project.path)
            .classpath(classpath)
            .jar(&console_launcher)
            .test_report(&manifest.project.out_paths().test_report)
            .args(&["--details", "tree"])
            .args(&["--exclude-engine", "junit-vintage"])
            .args(&["--exclude-engine", "junit-platform-suite"]);

        if let Ok(test_files) =
            util::paths::all_files_recursive(vec![], manifest.project.test.clone())
        {
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

impl<'a> Test<'_> {
    pub fn new(buildk: &'a BuildK, java: &'a Java) -> Test<'a> {
        Test { buildk, java }
    }
}
