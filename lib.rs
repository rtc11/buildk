use futures::lock::Mutex;
use itertools::Itertools;
use std::env::args;
use std::sync::Arc;

use terminal_spinners::{SpinnerBuilder, DOTS7};

use command::Command;
use manifest::config::Config;
use util::buildk_output::BuildkOutput;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //let timer = Timer::start();
    let config = Config::default();
    let command = Arc::new(Mutex::new(Command::new(&config).expect("kotlin expected")));

    let executions = args()
        .skip(1)
        .flat_map(command::Option::from)
        .unique()
        .map(|option| {
            let option = option.clone();
            let config = config.clone();
            let command = command.clone();
            async move { 
                let mut command = command.lock().await;
                execute(
                    &option.clone(), 
                    &config.clone(), 
                    &mut command,
                ).await
            }
        })
        .collect::<Vec<_>>();

    let executions = futures::future::join_all(executions).await;

    let errors = executions.iter()
        .filter(|output| output.get_status() != 0 )
        .filter_map(|output| output.get_stderr())
        .fold(String::new(), |errors, output| {
            format!("{errors}\n{output}")
        });

    let outputs = executions.iter()
        .filter(|output| output.get_status() == 0 )
        .filter_map(|output| output.get_stdout())
        .fold(String::new(), |outputs, output| {
            format!("{outputs}\n{output}")
        });

    if !errors.is_empty() {
        //println!("{} in {}", Conclusion::FAILED, timer.elapsed());
        println!("{errors}");
        println!("{config}");
        println!("");
    } else {
        //println!("{} in {}", Conclusion::SUCCESS, timer.elapsed());
    }

    if !outputs.is_empty() {
        println!("{outputs}");
        println!("");
    }

    Ok(())
}

async fn execute(option: &command::Option, config: &Config, command: &mut Command) -> BuildkOutput {
    let handle = SpinnerBuilder::new()
        .spinner(&DOTS7)
        .prefix(" ")
        .text(format!(" {option}"))
        .start();

    let output = match option {
        command::Option::Clean => command.clean(config),
        command::Option::Fetch => command.fetch(config).await,
        command::Option::BuildSrc => command.build_src(config),
        command::Option::BuildTest => command.build_test(config),
        command::Option::Test => command.run_tests(config),
        command::Option::Release => command.release(config),
        command::Option::Run => command.run(config),
        command::Option::Deps => command.deps(config),
        command::Option::BuildTree => command.build_tree(config),
        command::Option::Config => command.config(config),
        command::Option::Help => command.help(config),
    };

    handle.stop_and_clear();

    println!(
        "\r{:<6} {:<12} ▸ {}",
        output.conclusion(),
        option,
        output.elapsed()
    );

    output
}

