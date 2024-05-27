use manifest::{config::BuildK, Manifest};
use process::java::Java;
use util::buildk_output::BuildkOutput;

use crate::Command;

pub(crate) struct Run<'a> {
    buildk: &'a BuildK,
    java: &'a Java<'a>,
}

impl<'a> Command for Run<'a> {
    type Item = String;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("run");

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.buildk.manifest)
            .expect("no buildk.toml found.");

        let runtime_deps = manifest.runtime_deps;
        let runtime_paths = runtime_deps.pkgs.iter().map(|dep| dep.jar_absolute_path()).collect::<Vec<_>>();
        let platform_deps = manifest.compile_deps;
        let platform_paths = platform_deps.pkgs.iter().map(|dep| dep.jar_absolute_path()).collect::<Vec<_>>();

        let out_paths = &manifest.project.out_paths();
        let mut classpath = vec![&out_paths.src, &manifest.project.src];

        classpath.extend(runtime_paths.iter());
        classpath.extend(platform_paths.iter());

        let main = match arg {
            Some(class) => class.to_string() + "Kt",
            None => manifest.project.main.replace(".kt", "Kt"),
        };

        self.java
            .builder()
            .workdir(&manifest.project.path)
            .classpath(classpath)
            .main(main)
            .run(&mut output)
    }
}

impl<'a> Run<'_> {
    pub fn new(buildk: &'a BuildK, java: &'a Java) -> Run<'a> {
        Run { buildk, java }
    }
}
