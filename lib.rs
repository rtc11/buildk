use itertools::Itertools;
use std::env::args;

use terminal_spinners::{SpinnerBuilder, DOTS};

use command::Command;
use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::timer::Timer;
use util::Conclusion;

fn main() {
    let timer = Timer::start();
    let config = Config::default();
    let mut command = Command::new(&config).expect("kotlin expected");

    let errors = args()
        .skip(1)
        .flat_map(command::Option::from)
        .unique()
        .map(|option| execute(&option, &config, &mut command))
        .filter(|output| output.get_status() != 0)
        .filter_map(|output| output.get_stderr())
        .fold(String::new(), |errors, output| {
            format!("{errors}\n{output}")
        });

    if errors.is_empty() {
        println!("{} in {}", Conclusion::SUCCESS, timer.elapsed());
    } else {
        println!("{} in {}", Conclusion::FAILED, timer.elapsed());
        println!("{errors}");
        println!("{config}");
    }

    fn execute(option: &command::Option, config: &Config, command: &mut Command) -> BuildkOutput {
        let handle = SpinnerBuilder::new()
            .spinner(&DOTS)
            .prefix(" ")
            .text(format!(" {option}"))
            .start();

        let output = match option {
            command::Option::Clean => command.clean(config),
            command::Option::Fetch => command.fetch(config),
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
}
