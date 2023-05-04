use std::collections::HashSet;
use std::env::args;

use command::Command;
use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::Conclusion;
use util::timer::Timer;

fn main() {
    let timer = Timer::start();
    let config = Config::default();
    let mut command = Command::new(&config).expect("kotlin expected");
    let options = args().skip(1).flat_map(command::Option::from).collect::<HashSet<_>>();
    let errors = options.iter()
        .map(|option| execute(&option, &config, &mut command))
        .filter_map(|output| output.get_stderr())
        .fold(String::new(), |errors, output| format!("{errors}\n{output}"));

    if errors.is_empty() {
        println!("{} in {}", Conclusion::SUCCESS, timer.elapsed());
    } else {
        println!("{} in {}", Conclusion::FAILED, timer.elapsed());
        println!("{errors}");
        println!("{config}");
    }

    fn execute(option: &command::Option, config: &Config, command: &mut Command) -> BuildkOutput {
        let output = match option {
            command::Option::Clean => command.clean(config),
            command::Option::Fetch => command.fetch(config),
            command::Option::BuildSrc => command.build_src(config),
            command::Option::BuildTest => command.build_test(config),
            command::Option::Test => command.run_tests(config),
            command::Option::Release => command.release(config),
            command::Option::Run => command.run(config),
            command::Option::List => command.list(config),
            command::Option::Help => command.help(config),
        };
        println!("{:<6} {:<12} ▸ {}", output.conclusion(), option, output.elapsed());
        output
    }
}
