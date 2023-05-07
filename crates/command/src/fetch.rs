use std::thread::spawn;

use config::config::Config;
use config::dependencies::dependency::Dependency;
use http::client::Client;
use util::buildk_output::BuildkOutput;
use util::colorize::{Colors, OrderedColor};
use util::PartialConclusion;

use crate::Command;

impl Command {
    pub fn fetch(&mut self, config: &Config) -> BuildkOutput {
        let mut output = BuildkOutput::default();

        config.manifest.dependencies.iter().for_each(|dep| {
            match dep.is_cached() {
                true => {
                    print_status(dep, "[cached]", OrderedColor::Gray, 0);
                    output.conclude(PartialConclusion::CACHED);
                }
                false => match self.client.download(dep) {
                    Ok(_) => {
                        print_status(dep, "[fetched]", OrderedColor::Blue, 0);
                        output.conclude(PartialConclusion::SUCCESS);
                    }
                    Err(_) => {
                        print_status(dep, "[failed]", OrderedColor::Red, 0);
                        output.conclude(PartialConclusion::FAILED);
                    }
                }
            }
        });

        let mut handlers = vec![];

        config.manifest.dependencies.iter().for_each(|dep| {
            handlers.push(
                spawn({
                    let client = self.client.clone();
                    let dep = dep.clone();
                    move || {
                        download_missing_transitive_dependencies(client, dep.transitives(), 1);
                    }
                })
            );
        });

        for handler in handlers {
            let _ = handler.join();
        }

        output
    }
}

fn download_missing_transitive_dependencies(mut client: Client, transitives: Vec<Dependency>, depth: usize) {
    transitives.iter().for_each(|dep| {
        if !dep.is_cached() {
            match client.download(dep) {
                Ok(_) => print_status(dep, "[fetched]", OrderedColor::Blue, depth),
                Err(_) => print_status(dep, "[failed]", OrderedColor::Red, depth),
            }
            download_missing_transitive_dependencies(client.clone(), dep.transitives(), depth + 1);
        } else {
            print_status(dep, "[cached]", OrderedColor::Gray, depth);
        }
    });
}

fn print_status(dep: &Dependency, status: &str, color: OrderedColor, depth: usize) {
    let display = format!("{:>depth$}{:<14}{}", "", status, dep.name, depth = (depth * 2));
    println!("{}", display.colorize(&color))
}
