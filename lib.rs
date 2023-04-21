use std::env::args;

use anyhow::Result;

use cmd::Cmd;
use console::Conclusion;
use console::timer::Timer;
use fs::toml::Config;

fn main() {
    let timer = Timer::start();

    let config = fs::toml::read();
    println!("{}", config);

    let args = args();
    if args.len() == 1 { eprintln!("Missing command, e.g clean build") }

    let mut failures: String = String::default();
    let mut success: String = String::default();

    args.skip(1)
        .map(|arg| Cmd::try_from(arg).expect(""))
        .flat_map(|command| execute(command, &config))
        .for_each(|outcome| {
            match outcome {
                Ok(msg) => success = format!("{success}{msg}"),
                Err(err) => failures = format!("{failures}{err}"),
            }
        });

    println!("{failures}");
    println!("{success}");

    match failures.is_empty() {
        true => println!("{} in {}", Conclusion::SUCCESS, timer.elapsed()),
        false => println!("{} in {}", Conclusion::FAILED, timer.elapsed()),
    };
}

fn execute(command: Cmd, config: &Config) -> Vec<Result<String>> {
    match command {
        Cmd::Clean => cmd::clean(config),
        Cmd::Build => cmd::build(config),
        Cmd::Test => cmd::test(config),
        Cmd::Run => cmd::run(config),
        Cmd::Release => cmd::target(config),
    }
}
