use std::env::args;

use anyhow::{bail, Result};
use time::Instant;

use console::Colorize;
use kotlinc;
use kotlinc::Target;

fn main() {
    let start = Instant::now();
    let args = args();
    if args.len() == 1 { eprintln!("Missing command, e.g clean build") }

    let mut failures: String = String::default();
    let mut success: String = String::default();

    // TODO: multiple args and split_into_sub_commands will do things multiple times.
    args.skip(1)
        .flat_map(|command| split_into_sub_commands(command))
        .map(|command| execute(command) )
        .for_each(|outcome| {
            match outcome {
                Ok(msg) => {
                    println!(" {}", "✓".to_green());
                    success = format!("{success}{msg}");
                },
                Err(err) => {
                    println!(" {}", "✕".to_red());
                    failures = format!("{failures}{err}");
                }
            }
        });

    println!("{failures}");
    println!("{success}");

    match failures.is_empty() {
        true => println!("{} in {}", "COMPLETED".to_green(), start.elapsed()),
        false => println!("{} in {}", "FAILED".to_red(), start.elapsed()),
    };
}

// TODO: cache build steps with e.g. hashing files
fn split_into_sub_commands(command: String) -> Vec<String> {
    match command.as_str() {
        "clean" => vec!["clean".to_string()],
        "build" => vec!["build".to_string()],
        "test" => vec!["build".to_string(), "test".to_string()],
        "run" => vec!["build".to_string(), "run".to_string()],
        "release" => vec!["build".to_string(), "test".to_string(), "release".to_string()],
        _ => vec![command.to_string()]
    }
}

fn execute(command: String) -> Result<String> {
    print!("▸ {:<7}", &command);

    match command.as_str() {
        "clean" => kotlinc::clean(),
        "build" => kotlinc::build(),
        "test" => kotlinc::test(),
        "run" => kotlinc::run(),
        "release" => kotlinc::release(Target::App),
        _ => bail!("Invalid command {}", command),
    }
}
