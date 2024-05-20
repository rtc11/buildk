use std::collections::HashSet;

use async_std::task;
use http::client::{Client, DownloadResult};
use manifest::dependencies::{Kind, Name, Version};
use manifest::manifest::Manifest;
use manifest::{config::Config, dependencies::Dependency};
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::{deps, Command};

const DEBUG: bool = false;
const PRINT_DOWNLOADS: bool = false;

pub(crate) struct Fetch<'a> {
    config: &'a Config,
}

impl<'a> Command for Fetch<'a> {
    type Item = String;

    fn execute(&mut self, arg: Option<Self::Item>) -> BuildkOutput {
        let mut output = BuildkOutput::new("fetch");

        match arg {
            Some(artifact) => self.fetch_from_arg(&mut output, artifact),
            None => self.fetch_from_manifest(&mut output),
        }

        output
    }
}

impl<'a> Fetch<'a> {
    fn fetch_from_manifest(&mut self, output: &mut BuildkOutput) {
        let manifest = <Option<Manifest> as Clone>::clone(&self.config.manifest)
            .expect("no buildk.toml found.");

        let deps = &manifest.dependencies;
        self.fetch_deps(deps, output)
    }
}

impl<'a> Fetch<'a> {
    fn fetch_from_arg(&mut self, output: &mut BuildkOutput, artifact: String) {
        let (name, version) = artifact.into_name_version_touple();

        if let Ok(dep) = Dependency::new(Kind::Source, name, version) {
            self.fetch_dep(dep, output)
        }
    }
}

impl<'a> Fetch<'a> {
    pub fn new(config: &'a Config) -> Fetch<'a> {
        Fetch { config }
    }

    fn fetch_dep(&mut self, dep: Dependency, output: &mut BuildkOutput) {
        self.fetch_deps(&[dep], output)
    }

    fn fetch_deps(&mut self, deps: &[Dependency], output: &mut BuildkOutput) {
        let client = Client;

        let downloads = task::block_on(async {
            let all_deps = deps::find_dependent_deps(deps.to_vec(), vec![], 0, false).await;

            println!("\rtotal deps: {}", all_deps.len());

            all_deps
                .into_iter()
                .filter(|dep| {
                    match dep.is_cached() {
                        true => print_status(dep, "[cached]", Color::Green, 0),
                        false => print_status(dep, "[missing]", Color::Red, 0),
                    }
                    !dep.is_cached()
                })
                .map(|dep| {
                    let config = self.config.clone();
                    let client = client.clone();

                    task::block_on(async {
                        println!("{:<10} {:<16}:{:<26}", "downloading", dep.name, dep.version);
                        client.download_async(&dep, &config).await
                    })
                })
                .collect::<Vec<_>>()
        });

        downloads
            .iter()
            .filter_map(|download| match download {
                DownloadResult::Failed(err) => Some(err.to_owned()),
                _ => None,
            })
            .for_each(|err| {
                eprintln!("\n{}", &err);
                output.append_stderr(err);
            });

        if output.get_stderr().is_some() {
            output.conclude(PartialConclusion::FAILED);
        } else if downloads.iter().any(|d| d.is_downloaded()) {
            output.conclude(PartialConclusion::SUCCESS);
        } else {
            output.conclude(PartialConclusion::CACHED);
        }
    }
}

trait IntoNameAndVersion {
    fn into_name_version_touple(self) -> (Name, Version);
}

impl IntoNameAndVersion for String {
    fn into_name_version_touple(self) -> (Name, Version) {
        let artifact = self.split(':').collect::<Vec<_>>();
        if artifact.len() != 2 {
            panic!("artifact must be in format: <name>:<version>")
        }

        (Name::from(artifact[0]), Version::from(artifact[1]))
    }
}

#[allow(dead_code)]
fn get_all_deps<'a>(_config: &'a Config, dep: &'a Dependency) -> HashSet<Dependency> {
    let mut result = HashSet::new();
    if result.insert(dep) {
        let transitives = dep.transitives().iter().fold(Vec::new(), |mut acc, dep| {
            let transitives = get_all_deps(_config, dep);
            acc.extend(transitives);
            acc
        });

        let mut res = result.into_iter().cloned().collect::<HashSet<_>>();
        res.extend(transitives);
        return res;
    }

    result.into_iter().cloned().collect()
}

#[allow(dead_code)]
async fn print_download_res(dep: &Dependency, res: &DownloadResult) {
    if PRINT_DOWNLOADS {
        match res {
            DownloadResult::Downloaded => print_status(dep, "[downloaded]", Color::Gray, 0),
            DownloadResult::Exist => print_status(dep, "[cached]", Color::Yellow, 0),
            DownloadResult::Failed(err) => {
                print_status(dep, "[failed]", Color::Red, 0);
                if DEBUG {
                    println!("{err}");
                }
            }
        }
    }
}

#[allow(dead_code)]
fn print_status(dep: &Dependency, status: &str, color: Color, depth: usize) {
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
