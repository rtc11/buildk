use std::collections::HashSet;
use async_std::task;
use http::client::{DownloadResult, Client};
use itertools::Itertools;
use manifest::{config::Config, dependencies::Dependency};
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use spinners::{Spinner, Spinners};
use crate::{deps, Commands, FetchCmd};

const DEBUG: bool = false;
const PRINT_DOWNLOADS: bool = false;

impl FetchCmd for Commands {
    fn fetch(
        &mut self, 
        config: &Config,
    ) -> BuildkOutput {
        let mut output = BuildkOutput::new("fetch");
        let client = Client;
            
        let deps = &config.manifest.dependencies;

        let downloads = task::block_on(async {
            let all_deps = deps::find_dependent_deps(deps.to_vec(), vec![], 0, false).await;

            println!("\rtotal deps: {}", all_deps.len());

            all_deps
                .into_iter()
                .filter(|dep| !dep.is_cached())
                .map(|dep| {

                    //let dep = dep.clone();
                    let config = config.clone();
                    let client = client.clone();

                    task::block_on(async {
                        let mut spinner = Spinner::new(Spinners::Dots7, format!("\r          downloading {}:{}", dep.name, dep.version).to_string());
                        let download_res = client.download_async(&dep, &config).await;
                        spinner.stop();
                        //println!("downloaded {}:{}", dep.name, dep.version);
                        download_res
                    })
                }).collect_vec()
        });

        downloads
            .iter()
            .filter_map(|download | match download {
                DownloadResult::Failed(err) => Some(err.to_owned()),
                _ => None,
            }).for_each(|err| {
                output.append_stderr(err);
            });

        if output.get_stderr().is_some() {
            output.conclude(util::PartialConclusion::FAILED);
        } else if downloads.iter().any(|d| d.is_downloaded()) {
            output.conclude(util::PartialConclusion::SUCCESS);
        } else {
            output.conclude(util::PartialConclusion::CACHED);
        }

        output
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

