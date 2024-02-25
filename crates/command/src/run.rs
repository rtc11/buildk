use std::path::PathBuf;

use manifest::config::Config;
use manifest::dependencies::DependenciesTools;
use manifest::manifest::Manifest;
use process::java::Java;
use util::buildk_output::BuildkOutput;

use crate::Command;

pub (crate) struct Run<'a> {
    config: &'a Config,
    java: &'a Java<'a>,
}

impl <'a> Command for Run<'a> {
    type Item = String;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("run");

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest).expect("manifest");

        let runtime_deps = manifest.dependencies.src_deps();
        let runtime_paths = runtime_deps.iter().map(|dep| dep.jar_absolute_path()).collect::<Vec<PathBuf>>();
        let platform_deps = manifest.dependencies.platform_deps();
        let platform_paths = platform_deps.iter().map(|dep| dep.jar_absolute_path()).collect::<Vec<PathBuf>>();

        let mut classpath = vec![
            &manifest.project.out.src,
            &manifest.project.src,
        ];

        classpath.extend(runtime_paths.iter());
        classpath.extend(platform_paths.iter());

        let main = match arg {
            Some(class) => class.to_string() + "Kt",
            None => manifest.project.compiled_main_file()
        };

        self.java.builder()
            .workdir(&manifest.project.path)
            .classpath(classpath)
            .main(main)
            .run(&mut output)
    }
}

impl <'a> Run<'_> {
    pub fn new(config: &'a Config, java: &'a Java) -> Run<'a> {
        Run { config, java }
    }
}

