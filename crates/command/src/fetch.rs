use async_std::task;
use dependency::Package;
use http::client::{Client, DownloadResult};
use manifest::config::BuildK;
use manifest::Manifest;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion;

use crate::{deps, Command};

const DEBUG: bool = false;

pub(crate) struct Fetch<'a> {
    buildk: &'a BuildK,
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
        let manifest = <Option<Manifest> as Clone>::clone(&self.buildk.manifest)
            .expect("no buildk.toml found.");

        let deps = &manifest.all_packages.pkgs;
        self.fetch_deps(deps, output)
    }
}

impl<'a> Fetch<'a> {
    fn fetch_from_arg(&mut self, output: &mut BuildkOutput, artifact: String) {
        let (name, namespace, version) = name_namespace_version(artifact);
        let pkg = Package::new(name, namespace, version, dependency::PackageKind::Compile);
        self.fetch_dep(pkg, output)
    }
}

fn name_namespace_version(input: String) -> (String, Option<String>, String) {
    let artifact = input.split("=").collect::<Vec<&str>>();
    if artifact.len() != 2 {
        panic!(
            "unexpected artifact name. Missing artifact or version: <namespace>..<name>=<version>"
        );
    }

    let (artifact, version) = (artifact[0], artifact[1].to_string());

    let artifact = artifact.split("..").collect::<Vec<&str>>();
    let (name, namespace) = match artifact.len() {
        1 => {
            let name = artifact[0].to_string();
            (name, None)
        }
        2 => {
            let name = artifact[1].to_string();
            let namespace = artifact[0].to_string();
            (name, Some(namespace))
        }
        _ => panic!("unexpected artifact name"),
    };

    (name, namespace, version)
}

impl<'a> Fetch<'a> {
    pub fn new(buildk: &'a BuildK) -> Fetch<'a> {
        Fetch { buildk }
    }

    fn fetch_dep(&mut self, pkg: Package, output: &mut BuildkOutput) {
        self.fetch_deps(&[pkg], output)
    }

    fn fetch_deps(&mut self, deps: &[Package], output: &mut BuildkOutput) {
        let client = Client;

        let downloads = task::block_on(async {
            let all_deps = deps::find_dependent_deps(deps.to_vec(), vec![], 0, false).await;

            println!("\rtotal deps: {}", all_deps.len());

            all_deps
                .into_iter()
                .filter(|dep| {
                    if DEBUG {
                        match dep.is_cached() {
                            true => print_status(dep, "[cached]", Color::Green, 0),
                            false => print_status(dep, "[missing]", Color::Red, 0),
                        }
                    }
                    !dep.is_cached()
                })
                .map(|dep| {
                    let config = self.buildk.clone();
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

fn print_status(pkg: &Package, status: &str, color: Color, depth: usize) {
    let display = format!(
        "{:>depth$}{:<14}{}:{}",
        "",
        status,
        pkg.name,
        pkg.version,
        depth = (depth * 2),
    );
    println!("\r{}", display.colorize(&color))
}

/* fn get_all_deps<'a>(_config: &'a Config, dep: &'a Dependency) -> HashSet<Dependency> {
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
 */
