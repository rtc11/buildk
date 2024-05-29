// use std::path::Path;

use manifest::config::BuildK;
use manifest::Manifest;
use process::java::{Java, JavaBuilder};
use util::buildk_output::BuildkOutput;

// use crate::tree::HeaderKt;
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

        let test_deps = manifest
            .test_deps
            .pkgs
            .iter()
            // .filter(|pkg| pkg.name != "junit-platform-console-standalone")
            .map(|pkg| pkg.jar_absolute_path())
            .collect::<Vec<_>>();

        // let junit = manifest
        //     .test_deps
        //     .pkgs
        //     .iter()
        //     .find(|pkg| pkg.name == "junit-jupiter-api")
        //     .map(|pkg| pkg.jar_absolute_path())
        //     .expect("missing junit");

        let kotlin_stdlib = manifest.compile_deps
            .pkgs
            .iter()
            .find(|pkg| pkg.name == "kotlin-stdlib")
            .map(|pkg| pkg.jar_absolute_path())
            .expect("kotlin-stdlib");

        // let kotlin_stdlib = manifest
        //     .kotlin_home
        //     .unwrap()
        //     .join("libexec")
        //     .join("lib")
        //     .join("kotlin-stdlib.jar");

        let out_paths = &manifest.project.out_paths();
        let mut classpath = vec![&out_paths.src, &out_paths.test, &kotlin_stdlib];
        // let mut classpath = vec![&out_paths.src, &out_paths.test, &junit, &kotlin_stdlib];

        // TODO: working example: java -jar /Users/robin/.buildk/cache/org.junit.platform/junit-platform-console-standalone/1.10.2/pkg.jar -cp out/test:out/src:/Users/robin/.buildk/cache/org.jetbrains.kotlin/kotlin-stdlib/1.9.22/pkg.jar --scan-classpath --disable-banner --exclude-engine=junit-vintage --exclude-engine=junit-platform-suite
        // TODO: working example: java -cp /Users/robin/.buildk/cache/org.junit.platform/junit-platform-console-standalone/1.10.2/pkg.jar:out/test:out/src:/Users/robin/.buildk/cache/org.jetbrains.kotlin/kotlin-stdlib/1.9.22/pkg.jar org.junit.platform.console.ConsoleLauncher --scan-classpath --disable-banner --exclude-engine=junit-vintage --exclude-engine=junit-platform-suite

        classpath.extend(&test_deps);

        let mut java = self.java.builder();
        java.workdir(&manifest.project.path)
            .classpath(classpath);
        let java = self.junit5(&mut java);
        // let java = self.testng(&mut java);

        java.run(&mut output)
    }
}

impl<'a> Test<'_> {
    pub fn new(buildk: &'a BuildK, java: &'a Java) -> Test<'a> {
        Test { buildk, java }
    }

    #[allow(dead_code)]
    fn testng(&'a self, java: &'a mut JavaBuilder<'a>) -> &'a mut JavaBuilder {
        // let testng = manifest
        //     .test_deps
        //     .pkgs
        //     .iter()
        //     .find(|pkg| pkg.name == "testng")
        //     .map(|pkg| pkg.jar_absolute_path())
        //     .expect("missing testng");

        java.args(&["org.testng.TestNG", "testng.xml"])
    }

    #[allow(dead_code)]
    fn junit5(&'a self, java: &'a mut JavaBuilder<'a>) -> &'a mut JavaBuilder {
        let manifest = <Option<Manifest> as Clone>::clone(&self.buildk.manifest)
            .expect("no buildk.toml found.");

        java
            .args(&["org.junit.platform.console.ConsoleLauncher", "--scan-classpath"])
            .test_report(&manifest.project.out_paths().test_report)
            .args(&["--details", "tree", "--disable-banner"])
            .args(&["--exclude-engine", "junit-vintage"])
            .args(&["--exclude-engine", "junit-platform-suite"]);

        // if let Ok(test_files) =
        //     util::paths::all_files_recursive(vec![], manifest.project.test.clone())
        // {
        //     let test_packages = test_files
        //         .iter()
        //         .map(Path::new)
        //         .filter_map(|path| HeaderKt::parse(path).ok())
        //         .map(|it| it.package)
        //         .collect::<Vec<String>>();
        //
        //     for pkg in test_packages.iter() {
        //         java.args(&["--select-package", &pkg]);
        //     }
        // }

        java
    }
}
