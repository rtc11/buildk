use std::sync::{Mutex, Arc};
use std::thread;

use http::client::{Client, DownloadResult};
use manifest::config::Config;
use manifest::dependencies::Dependency;
use util::buildk_output::BuildkOutput;
use util::colorize::{Color, Colors};

use crate::Command;

const DEBUG: bool = true;
 
impl Command {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let output = BuildkOutput::default();
        let deps = config.manifest.dependencies.clone();
        let config = Arc::new(Mutex::new(config.clone()));

        let mut threads = vec![];
    
        for dep in deps {
            let config = config.clone();
            let client = self.client.clone();
            threads.push(thread::spawn({
                move || download_trans(
                    &client,
                    config,
                    dep.clone(),
                    0
                )
            }));
        }

        for thread in threads {
            let _ = thread.join();
        }

        output
    }
}

// TODO: find repeated dependencies
// TODO: add configuration option to set (override) version
fn download_trans(
    client: &Client, 
    config: Arc<Mutex<Config>>,
    dep: Dependency,
    depth: usize
) {
    match client.download(&dep, &config.clone().lock().unwrap()) { 
        DownloadResult::Downloaded => print_status(&dep, "[downloaded]", Color::Green, depth),
        DownloadResult::Exist => print_status(&dep, "[cached]", Color::Gray, depth),
        DownloadResult::Failed(_err) => print_status(&dep, "[failed]", Color::Red, depth),
    }
    let mut threads = vec![];
    let transitive_deps = dep.transitives();
    for dep in transitive_deps {
        let config = config.clone();
        let client = client.clone();
        threads.push(thread::spawn({
            move || download_trans(
                &client, 
                config,
                dep,
                depth+1
            )
        }));
    }

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

