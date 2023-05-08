use std::sync::{Arc, Mutex};
use std::thread::spawn;

use config::config::Config;
use config::dependencies::dependency::Dependency;
use http::client::Client;
use util::buildk_output::BuildkOutput;
use util::colorize::{Colors, OrderedColor};
use util::PartialConclusion::{CACHED, FAILED, SUCCESS};

use crate::Command;

impl Command {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let output = Arc::new(Mutex::new(BuildkOutput::default()));
        let mut handlers = vec![];

        config.manifest.dependencies.iter().for_each(|dep| {
            handlers.push(
                spawn({
                    let mut client = self.client.clone();
                    let dep = dep.clone();
                    let output = output.clone();
                    move || {
                        client.download_transitive(output, &dep, 0);
                    }
                })
            );
        });

        for handler in handlers {
            let _ = handler.join();
        }

        let output = output.lock().unwrap().clone();
        output
    }
}

trait DownloadTransitive {
    fn download_transitive(&mut self, output: Arc<Mutex<BuildkOutput>>, dep: &Dependency, depth: usize);
}

impl DownloadTransitive for Client {
    fn download_transitive(&mut self, output: Arc<Mutex<BuildkOutput>>, dep: &Dependency, depth: usize) {
        if !dep.is_cached() {
            match self.download(dep) {
                Ok(_) => {
                    output.lock().unwrap().conclude(SUCCESS);
                    print_status(dep, "[fetched]", OrderedColor::Blue, depth);
                }
                Err(e) => {
                    output.lock().unwrap().conclude(FAILED);
                    output.lock().unwrap().stderr(e.to_string());
                    print_status(dep, "[failed]", OrderedColor::Red, depth);
                }
            }

            let mut handlers = vec![];
            dep.transitives().iter().for_each(|dep| {
                handlers.push(
                    spawn({
                        let mut client = self.clone();
                        let dep = dep.clone();
                        let output = output.clone();
                        move || {
                            client.download_transitive(output, &dep, depth + 1);
                        }
                    })
                );
            });
            for handler in handlers {
                let _ = handler.join();
            }
        } else {
            output.lock().unwrap().conclude(CACHED);
            print_status(dep, "[cached]", OrderedColor::Gray, depth);
        }
    }
}

fn print_status(dep: &Dependency, status: &str, color: OrderedColor, depth: usize) {
    let display = format!("{:>depth$}{:<14}{}", "", status, dep.name, depth = (depth * 2));
    println!("{}", display.colorize(&color))
}
