use futures::future::BoxFuture;
use futures::FutureExt;
use termtree::Tree;

use manifest::config::Config;
use manifest::dependencies::Dependency;
use manifest::manifest::Manifest;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colorize, Colors};
use util::PartialConclusion;

use crate::Command;

pub(crate) struct Deps<'a> {
    config: &'a Config,
}

fn termtree_display(status: &str, dep: &Dependency) -> String {
    format!("{}:{} {}", dep.name, dep.version, status)
}

fn termtree_status(dep: &Dependency) -> String {
    match dep.is_cached() {
        true => "".as_green(),
        false => " ".as_red(),
    }
}

pub fn build_termtree(
    dependency: Dependency,
    mut traversed: Vec<Dependency>,
    depth: usize,
    limit: usize,
) -> anyhow::Result<(Tree<String>, Vec<Dependency>)> {
    match traversed.contains(&dependency) {
        true => anyhow::bail!("already processed"),
        false => traversed.push(dependency.clone())
    }

    let status = termtree_status(&dependency);
    let display = termtree_display(&status, &dependency);
    let color = Color::get_index(depth);
    let label = display.colorize(&color).to_string();

    if depth < limit {
        let (tree, traversed) = dependency
            .transitives()
            .into_iter()
            .filter(|it| !traversed.contains(it))
            .fold(
                (Tree::new(label), traversed.clone()),
                |(mut root, root_trav), entry| {
                    match build_termtree(entry, root_trav.clone(), depth + 1, limit) {
                        Ok((tree, traversed)) => {
                            root.push(tree);
                            (root, traversed)
                        }
                        Err(_) => {
                            (root, root_trav)
                        }
                    }
                });
        Ok((tree, traversed))
    } else {
        Ok((Tree::new(label), traversed))
    }
}

impl<'a> Command for Deps<'a> {
    type Item = usize;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("deps");

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest)
            .expect("no buildk.toml found.");

        let limit = arg.unwrap_or(999);

        if !manifest.dependencies.is_empty() {
            println!("{} found   {} missing", "".as_green(), " ".as_red());
        }

        let mut traversed = vec![];
        for dep in manifest.dependencies.iter() {
            let (tree, newly_traversed) = build_termtree(dep.clone(), traversed.clone(), 0, limit).unwrap();
            traversed = newly_traversed;
            print!("{}", tree);
        }

        // manifest.dependencies.iter().for_each(|dep| {
        //     let (tree, traversed) = build_termtree(dep.clone(), vec![], 0).unwrap();
        //     print!("{}", tree);
        // });

        match lsp::update_classpath(self.config) {
            Ok(_) => output.conclude(PartialConclusion::SUCCESS),
            Err(err) => output
                .conclude(PartialConclusion::FAILED)
                .stderr(err.to_string()),
        };

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
    use manifest::manifest::Manifest;

    /**
     * This function is used to update the classpath for the kotlin language server.
     **/
    pub(crate) fn update_classpath(config: &Config) -> anyhow::Result<()> {
        use std::fs::OpenOptions;
        use std::io::prelude::*;

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&config.manifest)
            .expect("no buildk.toml found.");

        let kls_classpath = home::home_dir()
            .map(|home| home.join(".config"))
            .expect("Failed to get home dir")
            .join("kotlin-language-server")
            .join("kls-classpath");

        let classpath = manifest.dependencies
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

#[cfg(test)]
mod tests {
    use manifest::dependencies::{Dependency, Kind, Name, Version};

    use crate::deps::build_termtree;

    #[test]
    fn test_termtree() -> anyhow::Result<()> {
        // let dep = Dependency::new(Kind::Source, Name::from("io.ktor.ktor-server-core"), Version::from("2.3.7"))?;
        let dep = Dependency::new(Kind::Source, Name::from("org.jetbrains.kotlin.kotlin-stdlib"), Version::from("1.9.22"))?;
        let (tree, _) = build_termtree(dep, vec![], 0, 1)?;

        println!("{tree}");
        Ok(())
    }
}
