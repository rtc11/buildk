use futures::future::BoxFuture;
use futures::FutureExt;
use termtree::Tree;

use manifest::config::Config;
use manifest::dependencies::Dependency;
use manifest::manifest::Manifest;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colorize, Colors};
use util::{PartialConclusion, DEBUG};

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

pub struct Counter {
    hit: usize,
    miss: usize,
}

impl Counter {
    fn hit() -> Self {
        Self { hit: 1, miss: 0 }
    }

    fn miss() -> Self {
        Self { hit: 0, miss: 1 }
    }

    fn apply(&mut self, other: Counter) {
        self.hit += other.hit;
        self.miss += other.miss;
    }
}

pub fn build_termtree(
    dep: Dependency,
    mut traversed: Vec<Dependency>,
    depth: usize,
    limit: usize,
) -> anyhow::Result<(Tree<String>, Vec<Dependency>, Counter)> {
    match traversed.contains(&dep) {
        true => anyhow::bail!("already processed"),
        false => traversed.push(dep.clone()),
    }

    if dep.name.to_string().contains("-bom") {
        anyhow::bail!("bom not supported yet");
    }

    let mut counter_acc = match dep.is_cached() {
        true => Counter::hit(),
        false => Counter::miss(),
    };

    let status = termtree_status(&dep);
    let display = termtree_display(&status, &dep);
    let color = Color::get_index(depth);
    let label = display.colorize(&color).to_string();

    if depth < limit {
        let (tree, traversed) = dep
            .transitives()
            .into_iter()
            .filter(|it| !traversed.contains(it))
            .fold(
                (Tree::new(label), traversed.clone()),
                |(mut tree_acc, trav_acc), entry| match build_termtree(
                    entry,
                    trav_acc.clone(),
                    depth + 1,
                    limit,
                ) {
                    Ok((tree, traversed, counter)) => {
                        tree_acc.push(tree);
                        counter_acc.apply(counter);
                        (tree_acc, traversed)
                    }
                    Err(_) => (tree_acc, trav_acc),
                },
            );
        Ok((tree, traversed, counter_acc))
    } else {
        Ok((Tree::new(label), traversed, counter_acc))
    }
}

pub fn acc_transitive_unique(dep: Dependency, mut traversed: Vec<Dependency>) -> Vec<Dependency> {
    match traversed.contains(&dep) {
        true => return traversed,
        false => traversed.push(dep.clone()),
    }

    let traversed = dep
        .transitives()
        .into_iter()
        .filter(|it| !traversed.contains(it))
        .filter(|it| it.jar_absolute_path().exists())
        .fold(traversed.clone(), |acc, entry| {
            acc_transitive_unique(entry, acc)
        });

    traversed
}

impl<'a> Command for Deps<'a> {
    type Item = usize;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("deps");

        // FIXME
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest)
            .expect("no buildk.toml found.");

        let limit = arg.unwrap_or(999);

        let mut traversed = vec![];
        let mut counter_acc = Counter { hit: 0, miss: 0 };
        for dep in manifest.dependencies.iter() {
            let (tree, newly_traversed, counter) =
                build_termtree(dep.clone(), traversed.clone(), 0, limit).unwrap();
            traversed = newly_traversed;
            counter_acc.apply(counter);
            print!("{}", tree);
        }

        if !manifest.dependencies.is_empty() {
            if counter_acc.hit > 0 {
                print!("found {} {}", "".as_green(), counter_acc.hit)
            }
            if counter_acc.miss > 0 {
                print!(" miss {} {}", " ".as_red(), counter_acc.miss)
            }
            println!("");
        }

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
            if DEBUG {
                println!("{}", stdout);
            }

            if !traversed.contains(dep) {
                traversed.push(dep.clone());
            }
        });

        let transitives = dependencies
            .iter()
            .flat_map(|it| it.transitives())
            .filter(|it| !traversed.contains(it))
            .collect::<Vec<_>>();

        find_dependent_deps(transitives, traversed, depth + 1, print).await
    }
    .boxed()
}

mod lsp {
    use std::os::unix::fs::OpenOptionsExt;

    use anyhow::Context;

    use manifest::config::Config;
    use manifest::manifest::Manifest;

    use crate::deps::acc_transitive_unique;

    /**
     * This function is used to update the classpath for the kotlin language server.
     **/
    pub(crate) fn update_classpath(config: &Config) -> anyhow::Result<()> {
        use std::fs::OpenOptions;
        use std::io::prelude::*;

        // TODO: add transitive dependencies to kls classpath

        // FIXME
        let manifest =
            <Option<Manifest> as Clone>::clone(&config.manifest).expect("no buildk.toml found.");

        let kls_classpath = home::home_dir()
            .map(|home| home.join(".config"))
            .expect("Failed to get home dir")
            .join("kotlin-language-server")
            .join("classpath"); // see https://github.com/fwcd/kotlin-language-server?tab=readme-ov-file#figuring-out-the-dependencies

        let deps = manifest
            .dependencies
            .iter()
            .fold(vec![], |acc, dep| acc_transitive_unique(dep.clone(), acc));

        /*
                let classpath = manifest
                    .dependencies
                    .iter()
                    .map(|dep| dep.jar_absolute_path().display().to_string())
                    .collect::<Vec<_>>()
                    .join(":");
        */

        let classpath = deps
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
        let dep = Dependency::new(
            Kind::Source,
            Name::from("org.jetbrains.kotlin.kotlin-stdlib"),
            Version::from("1.9.22"),
        )?;
        let (tree, _, counter) = build_termtree(dep, vec![], 0, 1)?;

        println!("{tree}");
        println!("hit: {} miss: {}", counter.hit, counter.miss);
        Ok(())
    }
}
