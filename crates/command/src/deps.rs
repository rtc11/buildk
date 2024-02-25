use async_std::task;
use futures::future::BoxFuture;
use futures::FutureExt;

use manifest::config::Config;
use manifest::dependencies::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::Command;

pub(crate) struct Deps<'a> {
    config: &'a Config,
}

impl<'a> Command for Deps<'a> {
    type Item = ();

    fn execute(&mut self, _arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("deps");

        match lsp::update_classpath(self.config) {
            Ok(_) => output.conclude(PartialConclusion::SUCCESS),
            Err(err) => output
                .conclude(PartialConclusion::FAILED)
                .stderr(err.to_string()),
        };

        task::block_on(async {
            let deps = self.config.manifest.dependencies.clone();
            let deps = find_dependent_deps(deps, vec![], 0, true).await;
            println!("deps contains: {:?}", deps.len());
        });

        output.conclude(PartialConclusion::SUCCESS);

        output.to_owned()
    }
}

impl<'a> Deps<'a> {
    pub fn new(config: &'a Config) -> Deps<'a> {
        Deps { config }
    }
}

fn status(dep: &Dependency) -> &str {
    match dep.is_cached() {
        true => "[cached]",
        false => "[missing]",
    }
}

fn display(status: &str, dep: &Dependency, depth: usize) -> String {
    format!(
        "\r{:>depth$}{:<14}{}:{}",
        "",
        status,
        dep.name,
        dep.version,
        depth = depth * 2
    )
}

pub fn find_dependent_deps(
    dependencies: Vec<Dependency>,
    mut traversed: Vec<Dependency>,
    depth: usize,
    print: bool,
) -> BoxFuture<'static, Vec<Dependency>> {
    async move {
        if dependencies.is_empty() {
            return traversed;
        }

        dependencies.iter().for_each(|dep| {
            let status = status(dep);
            let display = display(status, dep, depth);
            let color = Color::get_index(depth);
            let stdout = display.colorize(&color).to_string();
            if print {
                println!("{}", stdout);
            }
            traversed.push(dep.clone());
        });

        let transitives = dependencies
            .iter()
            .flat_map(|it| it.transitives())
            .filter(|it| !traversed.contains(it))
            .collect::<Vec<_>>();

        find_dependent_deps(
            transitives,
            traversed,
            depth + 1,
            print,
        ).await
    }.boxed()
}

mod lsp {
    use std::os::unix::fs::OpenOptionsExt;
    use anyhow::Context;
    use manifest::config::Config;

    /**
     * This function is used to update the classpath for the kotlin language server.
     **/
    pub(crate) fn update_classpath(config: &Config) -> anyhow::Result<()> {
        use std::fs::OpenOptions;
        use std::io::prelude::*;

        let kls_classpath = home::home_dir()
            .map(|home| home.join(".config"))
            .expect("Failed to get home dir")
            .join("kotlin-language-server")
            .join("kls-classpath");

        let classpath = config
            .manifest
            .dependencies
            .iter()
            .map(|dep| dep.jar_absolute_path().display().to_string())
            .collect::<Vec<_>>()
            .join(":");

        let file = OpenOptions::new()
            .mode(0o777)
            .write(true)
            .truncate(true)
            .open(&kls_classpath);

        let mut file = match file {
            Ok(file) => file,
            Err(_) => OpenOptions::new()
                .append(true)
                .create(true)
                .open(&kls_classpath)
                .with_context(|| format!("Failed to edit {}", &kls_classpath.display()))?,
        };

        write!(file, "#/bin/bash\necho {}", classpath).with_context(|| {
            format!(
                "Failed to write classpath to kotlin lsp file: {}",
                kls_classpath.display()
            )
        })?;

        Ok(())
    }
}
