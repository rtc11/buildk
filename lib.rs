use std::env::args;

use command::Command;
use config::config::Config;
use util::buildk_output::BuildkOutput;
use util::Conclusion;
use util::option::Option;
use util::timer::Timer;

fn main() {
    let timer = Timer::start();
    let config = Config::default();
    let mut command = Command::new(&config).expect("kotlin expected");
    let errors = args()
        .skip(1)
        .flat_map(Option::from)
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

    fn execute(option: &Option, config: &Config, command: &mut Command) -> BuildkOutput {
        let output = match option {
            Option::Clean => command.clean(config),
            Option::Fetch => command.fetch(config),
            Option::BuildSrc => command.build_src(config),
            Option::BuildTest => command.build_test(config),
            Option::Test => command.run_tests(config),
            Option::Release => command.release(config),
            Option::Run => {
                let run = command.run(config);
                if let Some(stdout) = run.get_stdout() {
                    println!("{stdout}");
                }
                run
            },
            Option::List => command.list(config),
            Option::Help => command.help(config),
        };
        println!("{:<6} {:<12} ▸ {}", output.conclusion(), option, output.elapsed());
        output
    }
}
