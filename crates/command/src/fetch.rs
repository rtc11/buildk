use std::sync::{Arc, Mutex};

use http::client::Client;
use manifest::config::Config;
use manifest::dependencies::Dependency;
use manifest::repositories::Repository;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};
use util::PartialConclusion::{CACHED, FAILED, SUCCESS};

use crate::Command;

const DEBUG: bool = true;

impl Command {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let output = Arc::new(Mutex::new(BuildkOutput::default()));

        parallel_fetch(
            &self.client,
            &output,
            config.manifest.repositories.clone(),
            &config.manifest.dependencies,
            0,
        );

        let output = output.lock().unwrap().clone();
        output
    }
}

// TODO: find repeated dependencies
// TODO: add configuration option to set (override) version
trait Transitives {
    fn download_transitive(
        &mut self,
        repos: &[Repository],
        output: Arc<Mutex<BuildkOutput>>,
        dep: &Dependency,
        depth: usize,
    );
}

impl Transitives for Client {
    fn download_transitive(
        &mut self,
        repos: &[Repository],
        output: Arc<Mutex<BuildkOutput>>,
        dep: &Dependency,
        depth: usize,
    ) {
        if !dep.is_cached() {
            match self.download(dep, repos) {
                Ok(_) => {
                    output.lock().unwrap().conclude(SUCCESS);
                    print_status(dep, "[fetched]", Color::Blue, depth);
                }
                Err(e) => {
                    output.lock().unwrap().conclude(FAILED);
                    output.lock().unwrap().stderr(e.to_string());
                    print_status(dep, "[failed]", Color::Red, depth);
                }
            }

            let dependencies = dep.transitives();
            parallel_fetch(self, &output, repos.to_vec(), &dependencies, depth + 1);
        } else {
            output.lock().unwrap().conclude(CACHED);
            print_status(dep, "[cached]", Color::Gray, depth);
        }
    }
}

fn parallel_fetch(
    client: &Client,
    output: &Arc<Mutex<BuildkOutput>>,
    repositories: Vec<Repository>,
    dependencies: &[Dependency],
    depth: usize,
) {
    let mut threads = vec![];
    
    dependencies.iter().for_each(|dep| {
        threads.push(std::thread::spawn({
            let mut client = client.clone();
            let dep = dep.clone();
            let output = output.clone();
            let repositories = repositories.clone();
            move || client.download_transitive(&repositories, output, &dep, depth)
        }));
    });

    for thread in threads {
        let _ = thread.join();
    }
}

fn print_status(dep: &Dependency, status: &str, color: Color, depth: usize) {
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
