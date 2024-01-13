use std::collections::HashSet;
use std::sync::Arc;
use futures::{future::BoxFuture, FutureExt, lock::Mutex};
use http::client::{Client, DownloadResult};
use manifest::{dependencies::Dependency, config::Config};
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};

use crate::Command;

const DEBUG: bool = true;

impl Command {
    /*
    pub async fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();
        let deps = config.manifest.dependencies.clone();
        let config = Arc::new(Mutex::new(config.clone()));
        let client = Arc::new(Mutex::new(self.client.clone()));

        let deps = deps
            .iter()
            .filter(|dep| !dep.is_cached())
            .collect::<Vec<_>>();


        // TODO: some error with transitive depenencies. This statement is not always true if transitive
        // dependencies are missing. 
        if deps.len() == 0 {
            output.conclude(util::PartialConclusion::CACHED);
        } else {
            for dep in deps {
                let config = config.clone();
                let client = clietlone();
                let dep = dep.clone();

                tokio::spawn(async move { 
                    download_transitive(
                        client,
                        config, 
                        &dep, 
                        0
                    ).await 
                });
            }

            // todo: add state to output
            output.conclude(util::PartialConclusion::SUCCESS);
        }

        output
    }
    */
    pub async fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let deps = &config.manifest.dependencies;

        deps.iter().for_each(|dep| {
            self.client.download_blocking(dep, config);
        });

        deps.iter().for_each(|dep| {
            print_status_blocking(&dep, "[directly]", Color::Gray, 0);
        });

        let mut all_deps: HashSet<Dependency> = HashSet::new();
        for dep in deps {
            let dep_with_transitives = get_all_deps(config, dep);
            all_deps.extend(dep_with_transitives);
        }
        all_deps.iter().for_each(|dep| {
            print_status_blocking(&dep, "[list]", Color::Gray, 0);
        });
        BuildkOutput::default()
    }
}

fn get_all_deps<'a>(
    config: &'a Config,
    dep: &'a Dependency,
) -> HashSet<Dependency> {
    let mut result = HashSet::new();
    if result.insert(dep) {
        let transitives = dep
            .transitives()
            .iter()
            .fold(Vec::new(), |mut acc, dep| {
                let transitives = get_all_deps(
                    &config,
                    &dep,
                );
                acc.extend(transitives);
                acc
            });


        let mut res = result.into_iter().cloned().collect::<HashSet<_>>();
        res.extend(transitives);
        return res
    }

    result.into_iter().cloned().collect()
}

async fn download(
    client: Arc<Mutex<Client>>,
    config: Arc<Mutex<Config>>,
    dep: &Dependency,
    depth: usize,
) {
    let config = config.clone();
    let dep = dep.clone();

    tokio::spawn(async move {
        let client = client.lock().await;
        let downloaded = client.download(dep.clone(), config).await;

        match downloaded {
            DownloadResult::Downloaded => print_status(&dep, "[downloaded]", Color::Green, depth).await, 
            DownloadResult::Exist => print_status(&dep, "[cached]", Color::Gray, depth).await,
            DownloadResult::Failed(_err) => {
                print_status(&dep, "[failed]", Color::Red, depth).await;
                if DEBUG {
                    println!("{_err}");
                }
            },
        }
    });
}

pub fn download_transitive<'a>(
    client: Arc<Mutex<Client>>,
    config: Arc<Mutex<Config>>,
    dep: &'a Dependency,
    depth: usize,
) -> BoxFuture<'a, anyhow::Result<()>> {
    async move {

        download(
            client.clone(),
            config.clone(),
            dep,
            depth
        ).await;

        let dependencies = dep.transitives().clone();
        dependencies.iter().for_each(|dep| {
            let client = client.clone();
            let config = config.clone();
            let dep = dep.clone();
            tokio::spawn(async move { 
                download_transitive(
                    client,
                    config,
                    &dep, 
                    depth + 1
                ).await
            });
        });

        Ok(())

    }.boxed()
}

fn print_status_blocking(dep: &Dependency, status: &str, color: Color, depth: usize) {
    if DEBUG {
        let display = format!(
            "{:>depth$}{:<14}{}:{}",
            "",
            status,
            dep.name,
            dep.version,
            depth = (depth * 2),
        );
        println!("\r{}", display.colorize(&color))
    }
}
async fn print_status(dep: &Dependency, status: &str, color: Color, depth: usize) {
    if DEBUG {
        let display = format!(
            "{:>depth$}{:<14}{}:{}",
            "",
            status,
            dep.name,
            dep.version,
            depth = (depth * 2),
        );
        println!("\r{}", display.colorize(&color))
    }
}

