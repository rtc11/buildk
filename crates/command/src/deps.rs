use manifest::config::Config;
use manifest::dependencies::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::Command;

const LIST_TRANSITIVE: bool = true;

impl Command {
    pub fn deps(&self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        match lsp::update_classpath(config) {
            Ok(_) => output.conclude(PartialConclusion::SUCCESS),

            Err(err) => output
                .conclude(PartialConclusion::FAILED)
                .stderr(err.to_string()),
        };

        let stdout = list_dependencies(&config.manifest.dependencies, vec![], 0);

        output
            .stdout(format!("{stdout}"))
            .conclude(PartialConclusion::SUCCESS);

        output.to_owned()
    }
}

mod lsp {
    use std::os::unix::fs::OpenOptionsExt;
    use anyhow::Context;
    use manifest::config::Config;

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

        write!(file, "#/bin/bash\necho {}", classpath)
            .with_context(|| {
                format!(
                    "Failed to write classpath to kotlin lsp file: {}",
                    kls_classpath.display()
                )
            })?;

        Ok(())
    }
}

fn list_dependencies<'a>(
    dependencies: &'a [Dependency],
    mut traversed: Vec<&'a str>,
    depth: usize,
) -> String {
    let color = Color::get_index(depth);

    let stdout = dependencies
        .iter()
        .map(|dep| {
            traversed.push(dep.path.as_str());

            let status = match dep.is_cached() {
                true => "[cached]",
                false => "[missing]",
            };

            let display = format!(
                "\r{:>depth$}{:<14}{}:{}",
                "",
                status,
                dep.name,
                dep.version,
                depth = depth * 2,
            );

            let stdout = format!("{}", display.colorize(&color));

            let transitives = &dep
                .transitives()
                .into_iter()
                .filter(|dep| !traversed.contains(&dep.path.as_str()))
                .collect::<Vec<_>>();

            let next = list_dependencies(transitives, traversed.clone(), depth + 1);

            if LIST_TRANSITIVE {
                format!("{}\n{}", stdout, next)
            } else {
                stdout
            }
        })
        .fold(String::new(), |acc, next| format!("{}{}", acc, next));

    stdout
}
